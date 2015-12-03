#define _GNU_SOURCE
#include <unordered_map>
#include <iostream>
#include <cstdlib>
#include <fstream>
#include <cstring>
#include <arpa/inet.h>
#include <netinet/in.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#if _MSC_VER
#include <Windows.h>
#else
#include <sys/time.h>
#endif
class IPLookup {
public:
    IPLookup() {
        std::memset(tbl24_, 0, sizeof(uint16_t) * TBL24_SIZE);
        std::memset(tblLong_, 0, sizeof(uint16_t) * TBL24_SIZE);
        currentTBLLong_ = 0;
    }

    void AddRoute(uint32_t address, uint16_t len, uint16_t nextHop) {
        prefixTable_[len][address] = nextHop;
    }

    void DeleteRoute(uint32_t address, uint16_t len) {
        prefixTable_[len].erase(address);
    }

    void ConstructFIB() {
        uint32_t entries = 0;
        uint32_t long_entries = 0;
        for (int i = 0; i <= 24; i++) {
            entries++;
            for (auto const& kv : prefixTable_[i]) {
                uint32_t addr = kv.first;
                uint16_t dest = kv.second;
                uint32_t start = addr >> 8;
                uint32_t end = start + (1u << (24 - i));
                for (uint32_t pfx = start; pfx < end; pfx++) {
                    tbl24_[pfx] = dest;
                }
            }
        }
        for (int i = 25; i <= 32; i++) {
            entries++;
            long_entries++;
            for (auto kv : prefixTable_[i]) {
                uint32_t addr = kv.first;
                uint16_t dest = kv.second;
                uint16_t tblDest = tbl24_[addr >> 8];
                if ((tblDest & OVERFLOW_MASK) == 0) {
                    uint32_t start = currentTBLLong_ + (addr & 0xff);
                    uint32_t end = start + (1u << (32 - i));
                    for (uint32_t j = currentTBLLong_;
                                  j < currentTBLLong_ + 256;
                                  j++) {
                        if (j < start || j >= end) {
                            tblLong_[j] = tblDest;
                        } else {
                            tblLong_[j] = dest;
                        }
                    }
                    tbl24_[addr >> 8] =
                        (uint16_t)((currentTBLLong_ >> 8) |
                                   OVERFLOW_MASK);
                    currentTBLLong_ += 256;
                } else {
                    uint32_t start = ((uint32_t)(tblDest & 
                                (~OVERFLOW_MASK)) << 8) +
                                     (addr & 0xff);
                    uint32_t end = start + (1u << (32 - i));
                    for (uint32_t j = start; j < end; j++) {
                        tblLong_[j] = dest;
                    }
                }
            }
        }
        std::cout << "Processed " << entries  << " entries and  " << long_entries << " long" << std::endl;
    }

    uint16_t RouteLookup(uint32_t ip) {
        uint16_t tblDest = tbl24_[ip >> 8];
        if ((tblDest & OVERFLOW_MASK) > 0) {
            uint32_t index = (((uint32_t)(tblDest & (~OVERFLOW_MASK)) << 8)
                               + (ip & 0xff));
            return tblLong_[index];
        } else {
            return tblDest;
        }
    }
private:
    static const size_t TBL24_SIZE = (1u << 24) + 1;
    static const uint16_t OVERFLOW_MASK = 0x8000;
    std::unordered_map<uint32_t, uint16_t> prefixTable_[33];
    uint16_t tbl24_[TBL24_SIZE];
    uint16_t tblLong_[TBL24_SIZE];
    uint32_t currentTBLLong_;
};

static uint64_t get_sec(void) {
#if _MSC_VER
	return timeGetTime() / 1000;
#else
	struct timeval time;
	gettimeofday(&time, NULL);
	//long millis = (time.tv_sec * 1000) + (time.tv_usec / 1000);
	return (uint64_t)time.tv_sec;
#endif
}

static uint32_t rand_fast() {
    static uint64_t seed = 0;
    seed = seed * 1103515245 + 12345;
    return (uint32_t)(seed >> 32);
}

void Benchmark(
      IPLookup* lookup,
      uint64_t warm,
      uint32_t batch,
      uint64_t batches) {
    lookup->ConstructFIB();
    uint64_t lastSec = get_sec();
    while (get_sec() - lastSec < warm) {
        for (uint32_t i = 0; i < batch; i++) {
            lookup->RouteLookup(rand_fast());
        }
    }
    uint32_t batchComputed = 0;
    lastSec = get_sec();
    uint64_t lastLookups = 0;
    uint32_t lastIP = 0;
    uint64_t dests = 0;
    while (batchComputed < batches) {
        for (int i = 0; i < batch; i++) {
            lastIP = rand_fast();
            dests = lookup->RouteLookup(lastIP);
            lastLookups++;
        }
        if (lastSec != get_sec()) {
            uint64_t currSec = get_sec();
            batchComputed++;
            std::cout << (currSec - lastSec) << " "
                      << batch << " "
                      << (lastLookups / (currSec - lastSec)) 
                      << " " << lastIP
                      << " " << dests << std::endl;
            lastSec = get_sec();
            lastLookups = 0;
        }
    }
}

int main(int argc, char* argv[]) {
    IPLookup *lookup = new IPLookup();
    if (argc < 2) {
        std::cout << "Usage: lookup rib" << std::endl;
        return 1;
    }
    std::ifstream rib(argv[1]);
    while (!rib.eof()) {
        char line[512];
        //std::string line;
        //std::getline(rib, line);
        rib.getline(line, 512);
        //char* asString = line.c_str();
        char* ipPart = std::strtok(line, " ");
        char* dest = std::strtok(NULL, " ");
        char* ip = std::strtok(ipPart, "/");
        char* len = std::strtok(NULL, "/");
        if (ipPart == NULL || dest == NULL || ip == NULL || len == NULL) {
            continue;
        }
        uint32_t ipInt = 0;
        int convert = inet_pton(AF_INET, ip, &ipInt);
        if (!convert) {
            std::cerr << "Error converting " << ip << std::endl;
        }
        ipInt = ntohl(ipInt);
        lookup->AddRoute(ipInt, atoi(len), atoi(dest));
    }
    const uint64_t WARM = 5;
    const uint32_t BATCH = 512;
    const uint64_t BATCHES = 1024;
    Benchmark(lookup, WARM, BATCH, BATCHES);
    //for (int bexp = 0; bexp < BATCH; bexp++) {
        //Benchmark(lookup, WARM, (1L << bexp), BATCHES);
    //}
}
