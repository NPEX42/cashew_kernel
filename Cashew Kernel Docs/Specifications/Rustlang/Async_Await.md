# Async Await
---
## Overview
- Syntactic Sugar Over A Finite-State Automata.
- Co-operative Multitasking

## Basic Syntax
---
```rust
fn really_long_function_sync() {
	// Really Complex, Slow Operation
	let a = 9 + 10; // 9 + 10 = 21
	
}

async fn really_long_function_async() {
	//Again, A REALLY Complex, Slow Operation
	let a = 420 / 69;
}

pub async fn main() {
	// Run really_long_function One Thousand Times.
	for _ in 0..1000 {
		really_long_function_sync();
	}

	// Run The Function, Switching Execution To Something Else after calling The
	//Function
	for _ in 0..1000 {
		await really_long_function_async();
	}
}
```
