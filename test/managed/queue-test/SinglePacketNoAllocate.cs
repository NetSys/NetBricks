using System;
using System.Collections.Concurrent;
using System.Threading;
using System.Diagnostics;

namespace E2D2 {
public class SinglePacketTestReuse {

  protected internal ConcurrentQueue<Packet> queue_;

  private int producerCore_;
  private int consumerCore_;
  public long received_;
  private Packet[] packets_;
  public SinglePacketTestReuse(int producerCore, int consumerCore, int nbufs) { 
    queue_ = new ConcurrentQueue<Packet>();
    producerCore_ = producerCore;
    consumerCore_ = consumerCore;
    received_ = 0;
    packets_ = new Packet[nbufs];
    for (int i = 0; i < nbufs; i++) {
      packets_[i] = new Packet(i);
    }
  }

  protected void ProducerStart() {
    SysUtils.SetAffinity(producerCore_);
    Stopwatch stopwatch = new Stopwatch();
    stopwatch.Start();
    long lastSec = SysUtils.GetSecond(stopwatch);
    long count = 0;
    long absCount = 0;
    while (true) {
      long currSec = SysUtils.GetSecond(stopwatch);
      Packet packet = packets_[(int)(absCount % packets_.Length)];
      Debug.Assert(packet != null);
      queue_.Enqueue(packet);
      count++;
      absCount++;
      if (currSec != lastSec) {
        lastSec = currSec;
        count = 0;
      }
    }
  }

  protected void ConsumerStart() {
    SysUtils.SetAffinity(consumerCore_);
    Stopwatch stopwatch = new Stopwatch();
    stopwatch.Start();
    long lastSec = SysUtils.GetSecond(stopwatch);
    long lastElapsed = stopwatch.ElapsedMilliseconds;
    long count = 0;
    while (true) {
      Packet pkt;
      if (queue_.TryDequeue(out pkt)) {
        count++;
        received_ = pkt.id;
      }
      long currSec = SysUtils.GetSecond(stopwatch);
      if (currSec != lastSec) {
        long currElapsed = stopwatch.ElapsedMilliseconds;
        Debug.Assert(currElapsed - lastElapsed > 0);
        Console.WriteLine(lastSec + " " + currSec + " " + SysUtils.GetCurrentCpu() + " " 
                        + count + " " + received_ + " " + (currElapsed - lastElapsed));
        lastSec = currSec;
        lastElapsed = currElapsed;
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
}
