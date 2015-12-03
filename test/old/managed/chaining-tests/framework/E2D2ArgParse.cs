using System;
using System.Collections.Generic;
using System.Runtime.CompilerServices; 
namespace E2D2 {
	public class E2D2Options {
		public int numRxq;
		public int numTxq;
		public int endIdx;
		public string  vportIn;
		public string vportOut;
		public E2D2Options() : this (1, 1, 0, "vport0", "vport0") {
		}
		public E2D2Options(int numRxq, int numTxq, int rest, string vportIn, string vportOut) {
			this.numRxq = numRxq;
			this.numTxq = numTxq;
			this.endIdx = rest;
			this.vportIn = vportIn;
			this.vportOut = vportOut;
		}
	}
	public sealed class E2D2OptionParser {
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static E2D2Options ParseOptions (string[] args) {

			int numRxq = 1;
			int numTxq = 1;
			int endIdx = args.Length;
			string vportIn = "vport0";
			string vportOut = "vport0";

			try {
				for (int i=0; i < args.Length; i++) {
					if (args[i] == "-r" || args[i] == "--rxq") {
						numRxq = Convert.ToInt32(args[i + 1]);
						i++;
					} else if (args[i] == "-t" || args[i] == "--txq") {
						numTxq = Convert.ToInt32(args[i+1]);
						i++;
					} else if (args[i] == "-i" || args[i] == "--vin") {
						vportIn = args[i+1];
						i++;
					} else if (args[i] == "-o" || args[i] == "--vout") {
						vportOut = args[i+1];
						i++;
					} else if (args[i] == "--") {
						endIdx = i + 1;
						break;
					}				
				}
			} catch (Exception e) {
				Console.WriteLine("Error parsing commandline " + e.Message);
			}
			return new E2D2Options(numRxq, numTxq, endIdx, vportIn, vportOut);
		}
	}
}
