#include <stdint.h>
// ========= File I/O ========= 
typedef char* cstr_t; 
typedef uint16_t file_t;

#define STDIN  0
#define STDOUT 1
#define STDERR 2

size_t write( file_t fd, char* buffer, size_t buffer_len );
size_t read ( file_t fd, char* buffer, size_t buffer_len );
file_t open ( cstr_t path, cstr_t options                );
void   close( file_t fd, char* buffer, size_t buffer_len );

// ========= Timers ========= 
typedef void (*timer_callback)(uint64_t);
typedef uint16_t timer_t;
typedef uint16_t timer_period_t; // Measured In 1/8192 ths Of A Second (122.0703125 us)
/*
    Register A Timer Callback, Returns A Timer ID
*/
timer_t register_timer_cb(timer_callback cb);
timer_period_t timer_period(timer_t timer);

void sleep(double seconds); 

