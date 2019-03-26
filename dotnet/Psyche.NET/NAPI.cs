using System;
using System.Runtime.InteropServices;

namespace Psyche
{
    public static class NAPI
    {
        private const string LibName = "psyche_capi";
        private const CallingConvention LibCall = CallingConvention.Cdecl;

        [UnmanagedFunctionPointer(LibCall)]
        public delegate void OnResultBytes(
            IntPtr context,
            IntPtr bytes,
            UIntPtr size
        );

        [UnmanagedFunctionPointer(LibCall)]
        public delegate void OnResultString(
            IntPtr context,
            [MarshalAs(UnmanagedType.LPStr)]
            string content
        );

        [UnmanagedFunctionPointer(LibCall)]
        public delegate void OnResultUids(
            IntPtr context,
            IntPtr uidsArray,
            UIntPtr count
        );

        [StructLayout(LayoutKind.Sequential)]
        public struct UID
        {
            private readonly byte field0;
            private readonly byte field1;
            private readonly byte field2;
            private readonly byte field3;
            private readonly byte field4;
            private readonly byte field5;
            private readonly byte field6;
            private readonly byte field7;
            private readonly byte field8;
            private readonly byte field9;
            private readonly byte field10;
            private readonly byte field11;
            private readonly byte field12;
            private readonly byte field13;
            private readonly byte field14;
            private readonly byte field15;
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct Opt<T> where T : struct
        {
            public bool HasValue => hasValue;
            public T Value => value;

            [MarshalAs(UnmanagedType.I1)]
            private readonly bool hasValue;
            private readonly T value;

            public Opt(bool hasValue, T value)
            {
                this.hasValue = hasValue;
                this.value = value;
            }

            public static Opt<T> Some(T value)
            {
                return new Opt<T>(true, value);
            }

            public static Opt<T> None()
            {
                return new Opt<T>(false, default(T));
            }
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct BrainBuilderConfig
        {
            public double PropagationSpeed;
            public double NeuronPotentialDecay;
            public double ActionPotentialTreshold;
            public double ReceptorsExcitation;
            public double ReceptorsInhibition;
            public double DefaultReceptorsMin;
            public double DefaultReceptorsMax;
            public double SynapseInactivityTime;
            public Opt<double> SynapseReconnectionRange;
            public Opt<double> SynapseOverdoseReceptors;
            public double SynapsePropagationDecay;
            public Opt<double> SynapseNewConnectionReceptors;
            public UIntPtr Neurons;
            public UIntPtr Connections;
            public double Radius;
            public double MinNeurogenesisRange;
            public double MaxNeurogenesisRange;
            public UIntPtr Sensors;
            public UIntPtr Effectors;
            public sbyte NoLoopConnections;
            public UIntPtr MaxConnectingTries;
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct OffspringBuilderConfig
        {
            public UIntPtr NewNeurons;
            public UIntPtr NewConnections;
            public double Radius;
            public double MinNeurogenesisRange;
            public double MaxNeurogenesisRange;
            public UIntPtr NewSensors;
            public UIntPtr NewEffectors;
            public sbyte NoLoopConnections;
            public UIntPtr MaxConnectingTries;
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct BrainActivityStats
        {
            public UIntPtr NeuronsCount;
            public UIntPtr SynapsesCount;
            public UIntPtr ImpulsesCount;
            public double NeuronsPotential;
            public double NeuronsPotentialMin;
            public double NeuronsPotentialMax;
            public double ImpulsesPotential;
            public double ImpulsesPotentialMin;
            public double ImpulsesPotentialMax;
            public double AllPotential;
            public double AllPotentialMin;
            public double AllPotentialMax;
            public UIntPtr IncomingNeuronConnectionsMin;
            public UIntPtr IncomingNeuronConnectionsMax;
            public UIntPtr OutgoingNeuronConnectionsMin;
            public UIntPtr OutgoingNeuronConnectionsMax;
            public double SynapsesReceptorsMin;
            public double SynapsesReceptorsMax;
        }

        [DllImport(LibName, CallingConvention = LibCall, EntryPoint = "psyche_default_brain_builder_config", CharSet = CharSet.Ansi)]
        public extern static void DefaultBrainBuilderConfig(ref BrainBuilderConfig config);

        [DllImport(LibName, CallingConvention = LibCall, EntryPoint = "psyche_default_offspring_builder_config", CharSet = CharSet.Ansi)]
        public extern static void DefaultOffspringBuilderConfig(ref OffspringBuilderConfig config);

        [DllImport(LibName, CallingConvention = LibCall, EntryPoint = "psyche_build_brain", CharSet = CharSet.Ansi)]
        public extern static UIntPtr BuildBrain(
            ref BrainBuilderConfig config
        );

        [DllImport(LibName, CallingConvention = LibCall, EntryPoint = "psyche_destroy_brain", CharSet = CharSet.Ansi)]
        public extern static void DestroyBrain(
            UIntPtr handle
        );

        [DllImport(LibName, CallingConvention = LibCall, EntryPoint = "psyche_has_brain", CharSet = CharSet.Ansi)]
        [return: MarshalAs(UnmanagedType.I1)]
        public extern static bool HasBrain(
            UIntPtr handle
        );

        [DllImport(LibName, CallingConvention = LibCall, EntryPoint = "psyche_process_brains", CharSet = CharSet.Ansi)]
        public extern static void ProcessBrains(
            double deltaTime
        );

        [DllImport(LibName, CallingConvention = LibCall, EntryPoint = "psyche_process_brain", CharSet = CharSet.Ansi)]
        [return: MarshalAs(UnmanagedType.I1)]
        public extern static bool ProcessBrain(
            UIntPtr handle,
            double deltaTime
        );

        [DllImport(LibName, CallingConvention = LibCall, EntryPoint = "psyche_serialize_bytes_brain", CharSet = CharSet.Ansi)]
        [return: MarshalAs(UnmanagedType.I1)]
        public extern static bool SerializeBytesBrain(
            UIntPtr handle,
            [MarshalAs(UnmanagedType.FunctionPtr)]
            OnResultBytes result,
            IntPtr resultContext
        );

        [DllImport(LibName, CallingConvention = LibCall, EntryPoint = "psyche_serialize_json_brain", CharSet = CharSet.Ansi)]
        [return: MarshalAs(UnmanagedType.I1)]
        public extern static bool SerializeJsonBrain(
            UIntPtr handle,
            [MarshalAs(UnmanagedType.I1)]
            bool pretty,
            [MarshalAs(UnmanagedType.FunctionPtr)]
            OnResultString result,
            IntPtr resultContext
        );

        [DllImport(LibName, CallingConvention = LibCall, EntryPoint = "psyche_serialize_yaml_brain", CharSet = CharSet.Ansi)]
        [return: MarshalAs(UnmanagedType.I1)]
        public extern static bool SerializeYamlBrain(
            UIntPtr handle,
            [MarshalAs(UnmanagedType.FunctionPtr)]
            OnResultString result,
            IntPtr resultContext
        );

        [DllImport(LibName, CallingConvention = LibCall, EntryPoint = "psyche_deserialize_bytes_brain", CharSet = CharSet.Ansi)]
        public extern static UIntPtr DeserializeBytesBrain(
            IntPtr bytes,
            UIntPtr size,
            [MarshalAs(UnmanagedType.I1)]
            bool killImpulses
        );

        [DllImport(LibName, CallingConvention = LibCall, EntryPoint = "psyche_deserialize_json_brain", CharSet = CharSet.Ansi)]
        public extern static UIntPtr DeserializeJsonBrain(
            [MarshalAs(UnmanagedType.LPStr)]
            string json,
            [MarshalAs(UnmanagedType.I1)]
            bool killImpulses
        );

        [DllImport(LibName, CallingConvention = LibCall, EntryPoint = "psyche_deserialize_yaml_brain", CharSet = CharSet.Ansi)]
        public extern static UIntPtr DeserializeYamlBrain(
            [MarshalAs(UnmanagedType.LPStr)]
            string yaml,
            [MarshalAs(UnmanagedType.I1)]
            bool killImpulses
        );

        [DllImport(LibName, CallingConvention = LibCall, EntryPoint = "psyche_brain_get_sensors", CharSet = CharSet.Ansi)]
        [return: MarshalAs(UnmanagedType.I1)]
        public extern static bool BrainGetSensors(
            UIntPtr handle,
            [MarshalAs(UnmanagedType.FunctionPtr)]
            OnResultUids result,
            IntPtr resultContext
        );

        [DllImport(LibName, CallingConvention = LibCall, EntryPoint = "psyche_brain_get_effectors", CharSet = CharSet.Ansi)]
        [return: MarshalAs(UnmanagedType.I1)]
        public extern static bool BrainGetEffectors(
            UIntPtr handle,
            [MarshalAs(UnmanagedType.FunctionPtr)]
            OnResultUids result,
            IntPtr resultContext
        );

        [DllImport(LibName, CallingConvention = LibCall, EntryPoint = "psyche_brain_sensor_trigger_impulse", CharSet = CharSet.Ansi)]
        [return: MarshalAs(UnmanagedType.I1)]
        public extern static bool BrainBrainSensorTriggerImpulse(
            UIntPtr handle,
            UID uid,
            double potential
        );

        [DllImport(LibName, CallingConvention = LibCall, EntryPoint = "psyche_brain_effector_potential_release", CharSet = CharSet.Ansi)]
        [return: MarshalAs(UnmanagedType.I1)]
        public extern static bool BrainBrainEffectorPotentialRelease(
            UIntPtr handle,
            UID uid,
            ref double outResult
        );

        [DllImport(LibName, CallingConvention = LibCall, EntryPoint = "psyche_offspring_mutated", CharSet = CharSet.Ansi)]
        public extern static UIntPtr BrainOffspringMutated(
            ref OffspringBuilderConfig config,
            UIntPtr handle
        );

        [DllImport(LibName, CallingConvention = LibCall, EntryPoint = "psyche_offspring_merged", CharSet = CharSet.Ansi)]
        public extern static UIntPtr BrainOffspringMerged(
            ref OffspringBuilderConfig config,
            UIntPtr handleA,
            UIntPtr handleB
        );

        [DllImport(LibName, CallingConvention = LibCall, EntryPoint = "psyche_get_brain_synapses_count", CharSet = CharSet.Ansi)]
        [return: MarshalAs(UnmanagedType.I1)]
        public extern static bool GetBrainSynapsesCount(
            UIntPtr handle,
            ref UIntPtr outResult
        );

        [DllImport(LibName, CallingConvention = LibCall, EntryPoint = "psyche_ignite_random_brain_synapses", CharSet = CharSet.Ansi)]
        [return: MarshalAs(UnmanagedType.I1)]
        public extern static bool IgniteRandomBrainSynapses(
            UIntPtr handle,
            UIntPtr count,
            double potentialMin,
            double potentialMax
        );

        [DllImport(LibName, CallingConvention = LibCall, EntryPoint = "psyche_brain_activity_stats", CharSet = CharSet.Ansi)]
        [return: MarshalAs(UnmanagedType.I1)]
        public extern static bool GetBrainActivityStats(
            UIntPtr handle,
            ref BrainActivityStats outResult
        );
    }
}
