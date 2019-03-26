using System;
using System.Runtime.InteropServices;

namespace Psyche
{
    public class Brain : IDisposable
    {
        private UIntPtr handle;

        public Brain(ref NAPI.BrainBuilderConfig config)
        {
            handle = NAPI.BuildBrain(ref config);
        }

        private Brain(UIntPtr handle)
        {
            this.handle = handle;
        }

        public void Dispose()
        {
            if (handle != UIntPtr.Zero)
            {
                NAPI.DestroyBrain(handle);
                handle = UIntPtr.Zero;
            }
        }

        public bool Exists()
        {
            return NAPI.HasBrain(handle);
        }

        public static void ProcessAll(double deltaTime)
        {
            NAPI.ProcessBrains(deltaTime);
        }

        public bool Process(double deltaTime)
        {
            return handle == UIntPtr.Zero
                ? false
                : NAPI.ProcessBrain(handle, deltaTime);
        }

        public byte[] SerializeBytes()
        {
            byte[] result = null;
            NAPI.SerializeBytesBrain(
                handle,
                (context, bytes, size) =>
                {
                    if (bytes != IntPtr.Zero && (int)size > 0)
                    {
                        result = new byte[(int)size];
                        Marshal.Copy(bytes, result, 0, (int)size);
                    }
                },
                IntPtr.Zero
            );
            return result;
        }

        public string SerializeJson(bool pretty = true)
        {
            string result = null;
            NAPI.SerializeJsonBrain(
                handle,
                pretty,
                (context, content) => result = content,
                IntPtr.Zero
            );
            return result;
        }

        public string SerializeYaml()
        {
            string result = null;
            NAPI.SerializeYamlBrain(
                handle,
                (context, content) => result = content,
                IntPtr.Zero
            );
            return result;
        }

        public static Brain DeserializeBytes(byte[] bytes, bool killImpulses = false)
        {
            var ptr = Marshal.AllocHGlobal(bytes.Length);
            Marshal.Copy(bytes, 0, ptr, bytes.Length);
            var handle = NAPI.DeserializeBytesBrain(ptr, (UIntPtr)bytes.Length, killImpulses);
            Marshal.FreeHGlobal(ptr);
            if (handle != UIntPtr.Zero)
            {
                return new Brain(handle);
            }
            return null;
        }

        public static Brain DeserializeJson(string json, bool killImpulses = false)
        {
            var handle = NAPI.DeserializeJsonBrain(json, killImpulses);
            if (handle != UIntPtr.Zero)
            {
                return new Brain(handle);
            }
            return null;
        }

        public static Brain DeserializeYaml(string yaml, bool killImpulses = false)
        {
            var handle = NAPI.DeserializeYamlBrain(yaml, killImpulses);
            if (handle != UIntPtr.Zero)
            {
                return new Brain(handle);
            }
            return null;
        }

        public NAPI.UID[] GetSensors()
        {
            NAPI.UID[] result = null;
            NAPI.BrainGetSensors(
                handle,
                (context, uids, count) =>
                {
                    if (uids != IntPtr.Zero && (uint)count > 0)
                    {
                        result = MakeUids(uids, count);
                    }
                },
                IntPtr.Zero
            );
            return result;
        }

        public NAPI.UID[] GetEffectors()
        {
            NAPI.UID[] result = null;
            NAPI.BrainGetEffectors(
                handle,
                (context, uids, count) =>
                {
                    if (uids != IntPtr.Zero && (uint)count > 0)
                    {
                        result = MakeUids(uids, count);
                    }
                },
                IntPtr.Zero
            );
            return result;
        }

        public bool SensorTriggerImpulse(NAPI.UID uid, double potential)
        {
            return NAPI.BrainBrainSensorTriggerImpulse(handle, uid, potential);
        }

        public bool EffectorPotentialRelease(NAPI.UID uid, out double potential)
        {
            var result = 0.0;
            if (NAPI.BrainBrainEffectorPotentialRelease(handle, uid, ref result))
            {
                potential = result;
                return true;
            }
            potential = result;
            return false;
        }

        public Brain OffspringMutated(ref NAPI.OffspringBuilderConfig config)
        {
            var handle = NAPI.BrainOffspringMutated(ref config, this.handle);
            if (handle != UIntPtr.Zero)
            {
                return new Brain(handle);
            }
            return null;
        }

        public static Brain OffspringMerged(ref NAPI.OffspringBuilderConfig config, Brain brainA, Brain brainB)
        {
            var handle = NAPI.BrainOffspringMerged(ref config, brainA.handle, brainB.handle);
            if (handle != UIntPtr.Zero)
            {
                return new Brain(handle);
            }
            return null;
        }

        public bool GetSynapsesCount(out UIntPtr count)
        {
            var result = UIntPtr.Zero;
            if (NAPI.GetBrainSynapsesCount(handle, ref result))
            {
                count = result;
                return true;
            }
            count = result;
            return false;
        }

        public bool IgniteRandomSynapses(UIntPtr count, double potentialMin, double potentialMax)
        {
            return NAPI.IgniteRandomBrainSynapses(handle, count, potentialMin, potentialMax);
        }

        public bool GetActivityStats(out NAPI.BrainActivityStats outResult)
        {
            var stats = new NAPI.BrainActivityStats();
            var status = NAPI.GetBrainActivityStats(handle, ref stats);
            outResult = stats;
            return status;
        }

        private NAPI.UID[] MakeUids(IntPtr array, UIntPtr count)
        {
            var result = new NAPI.UID[(uint)count];
            unsafe
            {
                var size = sizeof(NAPI.UID);
                for (var i = 0; i < (uint)count; ++i)
                {
                    result[i] = *(NAPI.UID*)array.ToPointer();
                    array += size;
                }
            }
            return result;
        }
    }
}
