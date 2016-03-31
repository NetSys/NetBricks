#include <rte_hash_crc.h>

// Make rte_hash_crc available to Rust. This adds some cost, will look into producing a pure Rust version.
uint32_t crc_hash_native(const void* data, uint32_t len, uint32_t initial) {
    return rte_hash_crc(data, len, initial);
}
