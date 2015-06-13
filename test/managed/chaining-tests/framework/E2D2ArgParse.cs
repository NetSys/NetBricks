using System;
using System.Collections.Generic;
using System.Runtime.CompilerServices; 
namespace E2D2 {
	public class E2D2Options {
		public int numRxq;
		public int numTxq;
		public int endIdx;
		public E2D2Options() : this (1, 1, 0) {
		}
		public E2D2Options(int _numRxq, int _numTxq, int _rest) {
			numRxq = _numRxq;
			numTxq = _numTxq;
			endIdx = _rest;
		}
	}
	public sealed class E2D2OptionParser {
        [MethodImpl(MethodImplOptions.AggressiveInlining)]
		public static E2D2Options ParseOptions (string[] args) {

			int numRxq = 1;
			int numTxq = 1;
			int endIdx = args.Length;

			Console.WriteLine("Creating option set");
			try {
				for (int i=0; i < args.Length; i++) {
					if (args[i] == "-r" || args[i] == "--rxq") {
						numRxq = Convert.ToInt32(args[i + 1]);
						i++;
					} else if (args[i] == "-t" || args[i] == "--txq") {
						numTxq = Convert.ToInt32(args[i+1]);
						i++;
					} else if (args[i] == "--") {
						endIdx = i;
						break;
					}				
				}
				Console.WriteLine("Done Parsing");
			} catch (Exception e) {
				Console.WriteLine("Error parsing commandline " + e.Message);
			}
			return new E2D2Options(numRxq, numTxq, endIdx);
		}
	}
}
