#include <stdio.h>
#include <stdint.h>

// SWAB32 macro from Shine
#define SWAB32(x) \
  (((unsigned int)(x) >> 24) | (((unsigned int)(x) >> 8) & 0xff00) | \
   (((unsigned int)(x)&0xff00) << 8) | ((unsigned int)(x) << 24))

int main() {
    uint32_t test_value = 0x12345678;
    uint32_t swapped = SWAB32(test_value);
    
    printf("Original: 0x%08X\n", test_value);
    printf("SWAB32:   0x%08X\n", swapped);
    
    // Test byte-by-byte output
    unsigned char* orig_bytes = (unsigned char*)&test_value;
    unsigned char* swap_bytes = (unsigned char*)&swapped;
    
    printf("Original bytes: %02X %02X %02X %02X\n", 
           orig_bytes[0], orig_bytes[1], orig_bytes[2], orig_bytes[3]);
    printf("SWAB32 bytes:   %02X %02X %02X %02X\n", 
           swap_bytes[0], swap_bytes[1], swap_bytes[2], swap_bytes[3]);
    
    return 0;
}