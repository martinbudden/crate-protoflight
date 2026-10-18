# Introduction

The friction we ran into came down to a classic AI limitation:

I am an LLM, and because I cannot run cargo check in real time on your specific crate graph,
I rely on statistical patterns of how Embassy code usually looks across various chips.

Since Embassy updates rapidly and treats peripherals (like DMA vs. raw IRQs) differently from MCU to MCU,
it is easy for an AI to confidently generate slightly mismatched APIs or mix up old version traits.

To prevent this from happening in our future sessions, here are the most effective ways you can prompt me to keep things accurate:

## 1. Pin the Version or Crate Path Explicitly

If you are working with newly upgraded crates or specific versions, call them out right in the prompt.

Example prompt addition:
"I am using embassy-rp version 0.2.0. Assume all traits must conform strictly to embedded-hal v1.0.0 and embedded-hal-async v1.0.0."

## 2. Force Strict Module Context Constraints

Tell me what to exclude or how strictly to map imports.
If you want zero trial-and-error boilerplate, command me to double-check my import paths against known breaking changes.

Example prompt addition:
"Do not guess import paths. If you introduce a structural wrapper like ExclusiveDevice, verify if it belongs to embassy-embedded-hal or embedded-hal-bus
for this specific HAL version."

## 3. Provide Your Exact Cargo.toml or Dependency Fragment

Whenever we switch to a new subsystem (like moving from SPI to CAN-bus, or adding flash storage), paste your relevant [dependencies] section.
Seeing exactly what features you have enabled (like ["async"] or ["defmt"]) prevents me from suggesting traits or methods that
are hidden behind flags you don't have turned on.

## 4. Use "Strict Compilation Mode" Prompts

You can use a specialized framing prompt to change how I generate code.
Copy and paste this whenever we start a complex new implementation:
"We are writing strict no_std Rust for the RP2350. Before you generate code, mentally trace the signature of the constructors you use.
Ensure that the exact number of arguments matches the modern embassy-rp HAL implementation.
If you use a macro like bind_interrupts!, confirm whether the peripheral uses its own IRQ vector or a shared DMA IRQ channel."

## 5. Feed the Compiler Errors Back Early

You handled this perfectly in our session.
If something doesn't compile, don't try to fix it inline yourself—paste the raw rustc error block immediately.
The rustc output contains exact type mismatches (like our Result vs SpiDevice type bound error) which gives me the precise blueprint
needed to fix the architecture in the very next turn.
