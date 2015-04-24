using System;
using System.Collections.Concurrent;
using System.Threading;
using System.Runtime.InteropServices;
using System.Diagnostics;
#if (!__MonoCS__)
using System.Linq;
#endif

namespace E2D2 {
public class Packet {
  Int64 id_;
  public Int64 id 
  {
      get {return id_;}
  }
  public Packet(Int64 id) {
    id_ = id;
  }
}

public class SinglePacketTest {
#if __MonoCS__
  [DllImport("libc.so.6")]
  private static extern int sched_getcpu();
    
  [DllImport("libc.so.6", SetLastError = true)]
  private static extern int sched_setaffinity(int pid, IntPtr cpusetsize, ulong[] cpuset);
#else
  [DllImport("kernel32.dll")]
  static extern int GetCurrentProcessorNumber();
#endif

  protected internal ConcurrentQueue<Packet> queue_;

  private int producerCore_;
  private int consumerCore_;
  private int nbufs_;
  public SinglePacketTest(int producerCore, int consumerCore, int nbuffers) { 
    queue_ = new ConcurrentQueue<Packet>();
    producerCore_ = producerCore;
    consumerCore_ = consumerCore;
    nbufs_ = nbuffers;
  }

  protected void SetAffinity (int core) {
    #if __MonoCS__
    ulong processorMask = 1UL << core;
    sched_setaffinity(0, new IntPtr(sizeof(ulong)), new [] {processorMask});
    #else
    long cpuMask = 1L << core;
    Thread.BeginThreadAffinity();
    #pragma warning disable 618
    int osThreadId = AppDomain.GetCurrentThreadId();
    #pragma warning restore 618
    ProcessThread thread =  Process.GetCurrentProcess().Threads.Cast<ProcessThread>()
                                       .Where(t => t.Id == osThreadId).Single();
    thread.ProcessorAffinity = new IntPtr(cpuMask);
    #endif
  }

  protected int GetCurrentCpu () {
    #if __MonoCS__
    return sched_getcpu();
    #else
    return GetCurrentProcessorNumber();
    #endif
  }

  protected void ProducerStart() {
    SetAffinity(producerCore_);
    //Packet[] packets = new Packet[nbufs_];
    Stopwatch stopwatch = new Stopwatch();
    stopwatch.Start();
    long lastSec = stopwatch.ElapsedMilliseconds / 1000;
    long count = 0;
    long absCount = 0;
    long buf = 0;
    while (true) {
      long currSec = stopwatch.ElapsedMilliseconds / 1000;
      queue_.Enqueue(new Packet(absCount));
      //buf = (buf + 1) % nbufs_;
      count++;
      absCount++;
      if (currSec != lastSec) {
        lastSec = currSec;
        // Console.WriteLine locks, so let us further reduce contention here
        count = 0;
      }
    }
  }

  protected void ConsumerStart() {
    SetAffinity(consumerCore_);
    Stopwatch stopwatch = new Stopwatch();
    stopwatch.Start();
    long lastSec = stopwatch.ElapsedMilliseconds / 1000;
    long count = 0;
    long received = 0;
    while (true) {
      Packet pkt;
      if (queue_.TryDequeue(out pkt)) {
        count++;
        received = pkt.id;
      }
      long currSec = stopwatch.ElapsedMilliseconds / 1000;
      if (currSec != lastSec) {
        lastSec = currSec;
        Console.WriteLine("Consume " + GetCurrentCpu() + " " + count + " " + received);
        count = 0;
      }
    }
  }

  public void Start() {
    Thread producer = new Thread(new ThreadStart(this.ProducerStart));
    Thread consumer = new Thread(new ThreadStart(this.ConsumerStart));
    producer.Start();
    consumer.Start();
    producer.Join();
    consumer.Join();
  }

}

public class QueueTest {
  public static void Main (string[] args) {
    #if __MonoCS__
    Console.WriteLine("Decided this is Mono");
    #else
    Console.WriteLine("Windows");
    #endif
    SinglePacketTest sp = new SinglePacketTest(0, 1, 100);
    sp.Start();
  }
}
}
