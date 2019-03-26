#include "pch.h"
#include <iostream>
#include <psyche_capi.hpp>
#include <vector>

void onGetUIDs(void* context, const UID* uids, uintptr_t count) {
	auto result = (std::vector<UID>*)context;
	for (auto i = 0; i < count; ++i) {
		result->push_back(uids[i]);
	}
}

void onGetYaml(void * context, const char* content) {
	auto result = (std::string*)context;
	result->assign(content);
}

int main() {
	BrainBuilderConfig brainConfig;
	psyche_default_brain_builder_config(&brainConfig);
	brainConfig.propagation_speed = 50;
	brainConfig.synapse_reconnection_range.has_value = true;
	brainConfig.synapse_reconnection_range.value = 15;
	brainConfig.neuron_potential_decay = 0.1;
	brainConfig.synapse_propagation_decay = 0.01;
	brainConfig.synapse_new_connection_receptors.has_value = true;
	brainConfig.synapse_new_connection_receptors.value = 2;
	brainConfig.neurons = 600;
	brainConfig.connections = 1000;
	brainConfig.min_neurogenesis_range = 5;
	brainConfig.max_neurogenesis_range = 15;
	brainConfig.radius = 50;
	brainConfig.sensors = 50;
	brainConfig.effectors = 25;

	auto brain = psyche_build_brain(&brainConfig);
	std::vector<UID> sensors;
	std::vector<UID> effectors;
	psyche_brain_get_sensors(brain, onGetUIDs, &sensors);
	psyche_brain_get_effectors(brain, onGetUIDs, &effectors);

	for (auto uid : sensors) {
		psyche_brain_sensor_trigger_impulse(brain, uid, 10);
	}

	std::string yaml;
	psyche_serialize_yaml_brain(brain, onGetYaml, &yaml);
	std::cout << "YAML:" << std::endl << yaml.c_str() << std::endl;
	brain = psyche_deserialize_yaml_brain(yaml.c_str(), false);

	auto running = true;
	while (running) {
		psyche_process_brain(brain, 1);

		std::cout << "======" << std::endl;
		for (auto uid : effectors) {
			auto potential = 0.0;
			if (psyche_brain_effector_potential_release(brain, uid, &potential)) {
				if (potential > 0) {
					running = false;
				}
				std::cout << potential << std::endl;
			}
			else {
				std::cout << "-" << std::endl;
			}
		}
	}

	uintptr_t synapses = 0;
	psyche_get_brain_synapses_count(brain, &synapses);
	psyche_ignite_random_brain_synapses(brain, synapses / 2, 1, 2);

	BrainActivityStats stats;
	psyche_brain_activity_stats(brain, &stats);
}
