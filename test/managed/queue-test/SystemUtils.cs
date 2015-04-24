using System;
using System.Threading;
using System.Runtime.InteropServices;
using System.Diagnostics;
#if (!__MonoCS__)
using System.Linq;
#endif

// System utilities for Queue Test
namespace E2D2 {
public class SysUtils {
#if __MonoCS__
  [DllImport("libc.so.6")]
  private static extern int sched_getcpu();
    
  [DllImport("libc.so.6", SetLastError = true)]
  private static extern int sched_setaffinity(int pid, IntPtr cpusetsize, ulong[] cpuset);
#else
  [DllImport("kernel32.dll")]
  private static extern int GetCurrentProcessorNumber();
#endif
  public static void SetAffinity (int core) {
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

  public static int GetCurrentCpu () {
    #if __MonoCS__
    return sched_getcpu();
    #else
    return GetCurrentProcessorNumber();
    #endif
  }

  public static long GetSecond (Stopwatch stopwatch) {
    stopwatch.ElapsedMilliseconds / 1000  
  }

}
}
