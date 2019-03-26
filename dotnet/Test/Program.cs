using Psyche;
using System;
using System.IO;

namespace Test
{
    class Program
    {
        static void Main(string[] args)
        {
            File.Copy("../../../../target/debug/psyche_capi.dll", "psyche_capi.dll", true);

            var brainConfig = new NAPI.BrainBuilderConfig();
            NAPI.DefaultBrainBuilderConfig(ref brainConfig);
            brainConfig.PropagationSpeed = 50;
            brainConfig.SynapseReconnectionRange = NAPI.Opt<double>.Some(15);
            brainConfig.NeuronPotentialDecay = 0.1;
            brainConfig.SynapsePropagationDecay = 0.01;
            brainConfig.SynapseNewConnectionReceptors = NAPI.Opt<double>.Some(2);
            brainConfig.Neurons = (UIntPtr)600;
            brainConfig.Connections = (UIntPtr)1000;
            brainConfig.MinNeurogenesisRange = 5;
            brainConfig.MaxNeurogenesisRange = 15;
            brainConfig.Radius = 50;
            brainConfig.Sensors = (UIntPtr)50;
            brainConfig.Effectors = (UIntPtr)25;

            var brain = new Brain(ref brainConfig);
            var sensors = brain.GetSensors();
            var effectors = brain.GetEffectors();

            foreach (var uid in sensors)
            {
                brain.SensorTriggerImpulse(uid, 10);
            }

            var yaml = brain.SerializeYaml();
            Console.WriteLine("YAML:");
            Console.WriteLine(yaml);
            brain = Brain.DeserializeYaml(yaml);

            var running = true;
            while (running)
            {
                brain.Process(1);

                Console.WriteLine("======");
                foreach (var uid in effectors)
                {
                    if (brain.EffectorPotentialRelease(uid, out var potential))
                    {
                        if (potential > 0)
                        {
                            running = false;
                        }
                        Console.WriteLine(potential);
                    }
                    else
                    {
                        Console.WriteLine("-");
                    }
                }
            }
            Console.ReadKey();
        }
    }
}
