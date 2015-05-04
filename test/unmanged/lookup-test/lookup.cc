#include <unordered_map>
#include <cstring>
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
        for (int i = 0; i <= 24; i++) {
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

int main(int argc, char* argv[]) {
    IPLookup lookup;
}
