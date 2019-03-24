extern crate libc;
extern crate psyche;
#[macro_use]
extern crate lazy_static;

use psyche::core::brain::Brain;
use psyche::core::brain_builder::BrainBuilder;
use psyche::core::config::Config;
use psyche::core::id::ID;
use psyche::core::offspring_builder::OffspringBuilder;
use psyche::serde::bytes::*;
use psyche::serde::json::*;
use psyche::serde::yaml::*;
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr::{copy_nonoverlapping, null, null_mut};
use std::sync::Mutex;

lazy_static! {
    static ref HANDLE_GEN: Mutex<Handle> = Mutex::new(0);
    static ref BRAINS: Mutex<HashMap<Handle, Brain>> = Mutex::new(HashMap::new());
}

pub type Handle = usize;
pub type Scalar = f64;

#[repr(C)]
pub struct UID([u8; 16]);

impl UID {
    pub fn from_id<T>(id: ID<T>) -> Self {
        Self(id.uuid().as_bytes().clone())
    }

    pub fn into_id<T>(&self) -> ID<T> {
        ID::from_bytes(self.0)
    }
}

#[repr(C)]
pub struct Opt<T> {
    pub has_value: bool,
    pub value: T,
}

impl<T> Opt<T> {
    pub fn new(has_value: bool, value: T) -> Self {
        Self { has_value, value }
    }

    pub fn some(value: T) -> Self {
        Self {
            has_value: true,
            value,
        }
    }

    pub fn none() -> Self
    where
        T: Default,
    {
        Self {
            has_value: false,
            value: T::default(),
        }
    }
}

impl<T> Opt<T> {
    pub fn into_option(&self) -> Option<T>
    where
        T: Clone,
    {
        if self.has_value {
            Some(self.value.clone())
        } else {
            None
        }
    }
}

#[repr(C)]
pub struct BrainBuilderConfig {
    pub propagation_speed: Scalar,
    pub neuron_potential_decay: Scalar,
    pub action_potential_treshold: Scalar,
    pub receptors_excitation: Scalar,
    pub receptors_inhibition: Scalar,
    pub default_receptors_min: Scalar,
    pub default_receptors_max: Scalar,
    pub synapse_inactivity_time: Scalar,
    pub synapse_reconnection_range: Opt<Scalar>,
    pub synapse_overdose_receptors: Opt<Scalar>,
    pub synapse_propagation_decay: Scalar,
    pub synapse_new_connection_receptors: Opt<Scalar>,
    pub neurons: usize,
    pub connections: usize,
    pub radius: Scalar,
    pub min_neurogenesis_range: Scalar,
    pub max_neurogenesis_range: Scalar,
    pub sensors: usize,
    pub effectors: usize,
    pub no_loop_connections: bool,
    pub max_connecting_tries: usize,
}

unsafe fn brain_builder_from_config(this: *const BrainBuilderConfig) -> BrainBuilder {
    let config = Config {
        propagation_speed: (*this).propagation_speed,
        neuron_potential_decay: (*this).neuron_potential_decay,
        action_potential_treshold: (*this).action_potential_treshold,
        receptors_excitation: (*this).receptors_excitation,
        receptors_inhibition: (*this).receptors_inhibition,
        default_receptors: (*this).default_receptors_min..(*this).default_receptors_max,
        synapse_inactivity_time: (*this).synapse_inactivity_time,
        synapse_reconnection_range: (*this).synapse_reconnection_range.into_option(),
        synapse_overdose_receptors: (*this).synapse_overdose_receptors.into_option(),
        synapse_propagation_decay: (*this).synapse_propagation_decay,
        synapse_new_connection_receptors: (*this).synapse_new_connection_receptors.into_option(),
    };
    BrainBuilder::new()
        .config(config)
        .neurons((*this).neurons)
        .connections((*this).connections)
        .radius((*this).radius)
        .min_neurogenesis_range((*this).min_neurogenesis_range)
        .max_neurogenesis_range((*this).max_neurogenesis_range)
        .sensors((*this).sensors)
        .effectors((*this).effectors)
        .no_loop_connections((*this).no_loop_connections)
        .max_connecting_tries((*this).max_connecting_tries)
}

impl Default for BrainBuilderConfig {
    fn default() -> Self {
        Self {
            propagation_speed: 1.0,
            neuron_potential_decay: 1.0,
            action_potential_treshold: 1.0,
            receptors_excitation: 1.0,
            receptors_inhibition: 0.05,
            default_receptors_min: 0.5,
            default_receptors_max: 1.5,
            synapse_inactivity_time: 0.05,
            synapse_reconnection_range: Opt::none(),
            synapse_overdose_receptors: Opt::none(),
            synapse_propagation_decay: 0.0,
            synapse_new_connection_receptors: Opt::none(),
            neurons: 100,
            connections: 0,
            radius: 10.0,
            min_neurogenesis_range: 0.1,
            max_neurogenesis_range: 1.0,
            sensors: 1,
            effectors: 1,
            no_loop_connections: true,
            max_connecting_tries: 10,
        }
    }
}

#[repr(C)]
pub struct OffspringBuilderConfig {
    pub new_neurons: usize,
    pub new_connections: usize,
    pub radius: Scalar,
    pub min_neurogenesis_range: Scalar,
    pub max_neurogenesis_range: Scalar,
    pub new_sensors: usize,
    pub new_effectors: usize,
    pub no_loop_connections: bool,
    pub max_connecting_tries: usize,
}

unsafe fn offspring_builder_from_config(this: *const OffspringBuilderConfig) -> OffspringBuilder {
    OffspringBuilder::new()
        .new_neurons((*this).new_neurons)
        .new_connections((*this).new_connections)
        .radius((*this).radius)
        .min_neurogenesis_range((*this).min_neurogenesis_range)
        .max_neurogenesis_range((*this).max_neurogenesis_range)
        .new_sensors((*this).new_sensors)
        .new_effectors((*this).new_effectors)
        .no_loop_connections((*this).no_loop_connections)
        .max_connecting_tries((*this).max_connecting_tries)
}

impl Default for OffspringBuilderConfig {
    fn default() -> Self {
        Self {
            new_neurons: 1,
            new_connections: 1,
            radius: 10.0,
            min_neurogenesis_range: 0.1,
            max_neurogenesis_range: 1.0,
            new_sensors: 1,
            new_effectors: 1,
            no_loop_connections: true,
            max_connecting_tries: 10,
        }
    }
}

#[no_mangle]
pub extern "C" fn psyche_default_brain_builder_config() -> BrainBuilderConfig {
    Default::default()
}

#[no_mangle]
pub extern "C" fn psyche_default_offspring_builder_config() -> OffspringBuilderConfig {
    Default::default()
}

#[no_mangle]
pub unsafe extern "C" fn psyche_build_brain(config: *const BrainBuilderConfig) -> Handle {
    if config.is_null() {
        return 0;
    }
    let brain = brain_builder_from_config(config).build();
    let handle = {
        let mut gen = HANDLE_GEN.lock().unwrap();
        let handle = *gen + 1;
        *gen = handle;
        handle
    };
    BRAINS.lock().unwrap().insert(handle, brain);
    handle
}

#[no_mangle]
pub extern "C" fn psyche_destroy_brain(handle: Handle) {
    BRAINS.lock().unwrap().remove(&handle);
}

#[no_mangle]
pub extern "C" fn psyche_has_brain(handle: Handle) -> bool {
    BRAINS.lock().unwrap().contains_key(&handle)
}

#[no_mangle]
pub extern "C" fn psyche_process_brains(delta_time: Scalar) {
    let mut brains = BRAINS.lock().unwrap();
    for brain in brains.values_mut() {
        drop(brain.process(delta_time));
    }
}

#[no_mangle]
pub extern "C" fn psyche_process_brain(handle: Handle, delta_time: Scalar) -> bool {
    if let Some(brain) = BRAINS.lock().unwrap().get_mut(&handle) {
        brain.process(delta_time).is_ok()
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn psyche_serialize_bytes_brain(
    handle: Handle,
    result: fn(*mut libc::c_void, *const libc::c_uchar, usize),
    result_context: *mut libc::c_void,
) -> bool {
    if let Some(brain) = BRAINS.lock().unwrap().get(&handle) {
        if let Ok(bytes) = brain_to_bytes(brain) {
            result(result_context, bytes.as_ptr(), bytes.len());
            return true;
        }
    }
    result(null_mut(), null(), 0);
    false
}

#[no_mangle]
pub extern "C" fn psyche_serialize_json_brain(
    handle: Handle,
    pretty: bool,
    result: fn(*mut libc::c_void, *const libc::c_char),
    result_context: *mut libc::c_void,
) -> bool {
    if let Some(brain) = BRAINS.lock().unwrap().get(&handle) {
        if let Ok(json) = brain_to_json(brain, pretty) {
            let json = CString::new(json).unwrap();
            result(result_context, json.as_ptr());
            return true;
        }
    }
    result(null_mut(), null());
    false
}

#[no_mangle]
pub extern "C" fn psyche_serialize_yaml_brain(
    handle: Handle,
    result: fn(*mut libc::c_void, *const libc::c_char),
    result_context: *mut libc::c_void,
) -> bool {
    if let Some(brain) = BRAINS.lock().unwrap().get(&handle) {
        if let Ok(yaml) = brain_to_yaml(brain) {
            let yaml = CString::new(yaml).unwrap();
            result(result_context, yaml.as_ptr());
            return true;
        }
    }
    result(null_mut(), null());
    false
}

#[no_mangle]
pub extern "C" fn psyche_deserialize_bytes_brain(
    bytes: *const libc::c_uchar,
    size: usize,
    kill_impulses: bool,
) -> Handle {
    let bytes = bytes_from_raw(bytes, size);
    if let Ok(mut brain) = brain_from_bytes(&bytes) {
        if kill_impulses {
            brain.kill_synapses_impulses();
        }
        let handle = {
            let mut gen = HANDLE_GEN.lock().unwrap();
            let handle = *gen + 1;
            *gen = handle;
            handle
        };
        BRAINS.lock().unwrap().insert(handle, brain);
        handle
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn psyche_deserialize_json_brain(
    json: *const libc::c_char,
    kill_impulses: bool,
) -> Handle {
    let json = string_from_raw_unsized(json as *const libc::c_uchar);
    if let Ok(mut brain) = brain_from_json(&json) {
        if kill_impulses {
            brain.kill_synapses_impulses();
        }
        let handle = {
            let mut gen = HANDLE_GEN.lock().unwrap();
            let handle = *gen + 1;
            *gen = handle;
            handle
        };
        BRAINS.lock().unwrap().insert(handle, brain);
        handle
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn psyche_deserialize_yaml_brain(
    yaml: *const libc::c_char,
    kill_impulses: bool,
) -> Handle {
    let yaml = string_from_raw_unsized(yaml as *const libc::c_uchar);
    if let Ok(mut brain) = brain_from_yaml(&yaml) {
        if kill_impulses {
            brain.kill_synapses_impulses();
        }
        let handle = {
            let mut gen = HANDLE_GEN.lock().unwrap();
            let handle = *gen + 1;
            *gen = handle;
            handle
        };
        BRAINS.lock().unwrap().insert(handle, brain);
        handle
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn psyche_brain_get_sensors(
    handle: Handle,
    result: fn(*mut libc::c_void, *const UID, usize),
    result_context: *mut libc::c_void,
) -> bool {
    if let Some(brain) = BRAINS.lock().unwrap().get(&handle) {
        let uids = brain
            .get_sensors()
            .iter()
            .map(|id| UID::from_id(*id))
            .collect::<Vec<_>>();
        result(result_context, uids.as_ptr(), uids.len());
        return true;
    }
    result(null_mut(), null(), 0);
    false
}

#[no_mangle]
pub extern "C" fn psyche_brain_get_effectors(
    handle: Handle,
    result: fn(*mut libc::c_void, *const UID, usize),
    result_context: *mut libc::c_void,
) -> bool {
    if let Some(brain) = BRAINS.lock().unwrap().get(&handle) {
        let uids = brain
            .get_effectors()
            .iter()
            .map(|id| UID::from_id(*id))
            .collect::<Vec<_>>();
        result(result_context, uids.as_ptr(), uids.len());
        return true;
    }
    result(null_mut(), null(), 0);
    false
}

#[no_mangle]
pub extern "C" fn psyche_brain_sensor_trigger_impulse(
    handle: Handle,
    uid: UID,
    potential: Scalar,
) -> bool {
    if let Some(brain) = BRAINS.lock().unwrap().get_mut(&handle) {
        brain
            .sensor_trigger_impulse(uid.into_id(), potential)
            .is_ok()
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn psyche_brain_effector_potential_release(handle: Handle, uid: UID) -> Opt<Scalar> {
    if let Some(brain) = BRAINS.lock().unwrap().get_mut(&handle) {
        if let Ok(potential) = brain.effector_potential_release(uid.into_id()) {
            return Opt::some(potential);
        }
    }
    Opt::none()
}

#[no_mangle]
pub unsafe extern "C" fn psyche_offspring_mutated(
    config: *const OffspringBuilderConfig,
    handle: Handle,
) -> Handle {
    if !config.is_null() {
        return 0;
    }
    if let Some(brain) = BRAINS.lock().unwrap().get(&handle) {
        let brain = offspring_builder_from_config(config).build_mutated(brain);
        let handle = {
            let mut gen = HANDLE_GEN.lock().unwrap();
            let handle = *gen + 1;
            *gen = handle;
            handle
        };
        BRAINS.lock().unwrap().insert(handle, brain);
        handle
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn psyche_offspring_merged(
    config: *const OffspringBuilderConfig,
    handle_a: Handle,
    handle_b: Handle,
) -> Handle {
    if !config.is_null() {
        return 0;
    }
    let brains = BRAINS.lock().unwrap();
    if let Some(brain_a) = brains.get(&handle_a) {
        if let Some(brain_b) = brains.get(&handle_b) {
            let brain = offspring_builder_from_config(config).build_merged(brain_a, brain_b);
            let handle = {
                let mut gen = HANDLE_GEN.lock().unwrap();
                let handle = *gen + 1;
                *gen = handle;
                handle
            };
            BRAINS.lock().unwrap().insert(handle, brain);
            return handle;
        }
    }
    0
}

fn bytes_from_raw(source: *const libc::c_uchar, size: usize) -> Vec<u8> {
    if source.is_null() || size == 0 {
        return vec![];
    }
    let mut result = vec![0; size];
    let target = result.as_mut_ptr();
    unsafe { copy_nonoverlapping(source, target, size) };
    result
}

fn string_from_raw_unsized(mut source: *const libc::c_uchar) -> String {
    if source.is_null() {
        return "".to_owned();
    }
    let mut bytes = vec![];
    unsafe {
        while *source != 0 {
            bytes.push(*source);
            source = source.add(1);
        }
    }
    let cstring = unsafe { CString::from_vec_unchecked(bytes) };
    cstring.into_string().unwrap()
}
