#include <stdio.h>
#include <assert.h>
#include <stdint.h>
#include <dpdk.h>

uint64_t fib (uint64_t l) {
	uint64_t a = 0, b = 1;
	while (b < l) {
		int temp = a;
		a = b;
		b = a + temp;
	}
	return b;
}
int main (int argc, char* argv[]) {

	int ret = init_system(12, 1);
	assert(ret >= 0);
	uint64_t f = fib(1ul << 48);
	printf("%lu\n", f);
}
