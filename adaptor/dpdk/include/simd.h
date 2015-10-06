#ifndef _SIMD_H_
#define _SIMD_H_

#include <stdio.h>

#include <x86intrin.h>

#define __xmm_aligned __attribute__((aligned(16)))
#define __ymm_aligned __attribute__((aligned(32)))
#define __zmm_aligned __attribute__((aligned(64)))

#if !__SSSE3__
  #error CPU must be at least Core 2 or equivalent (SSSE3 required)
#endif

static inline void print_m128i(__m128i a)
{
	uint32_t b[4] __xmm_aligned;
	
	*((__m128i *) b) = a;
	printf("%08x %08x %08x %08x\n", b[0], b[1], b[2], b[3]);
}

static inline __m128i gather_m128i(void *a, void *b)
{
#if 1
	/* faster (in a tight loop test. sometimes slower...) */
	__m128i t = _mm_loadl_epi64((__m128i *)a);
	return (__m128i)_mm_loadh_pd((__m128d)t, (double *)b);
#else
	return _mm_set_epi64x(*((uint64_t *)b), *((uint64_t *)a));
#endif
}

#if __AVX__

static inline void print_m256i(__m256i a)
{
	uint32_t b[8] __ymm_aligned;
	
	*((__m256i *) b) = a;
	printf("%08x %08x %08x %08x %08x %08x %08x %08x\n", 
			b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]);
}

static inline __m256d concat_two_m128d(__m128d a, __m128d b)
{
#if 1
	/* faster */
	return _mm256_insertf128_pd(
			_mm256_castpd128_pd256(a), b, 1);
#else
	return _mm256_permute2f128_si256(
			_mm256_castsi128_si256(a),
			_mm256_castsi128_si256(b), 
			(2 << 4) | 0);
#endif
}

#endif /* __AVX__ */

#endif
