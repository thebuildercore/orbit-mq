OrbitMQ is a smart traffic controller for satellites that makes sure emergency signals jump to the front of the line without wasting the satellite's limited computer power.

Here is the most intuitive way to understand what i built, without any engineering jargon:

Imagine a satellite is a tiny, flying post office in space. It is constantly generating thousands of letters—some are boring ("the battery is at 98%"), and some are massive emergencies ("we are about to crash into space junk"). But this post office has a terrible, slow internet connection to Earth.

Here is exactly what my software does to fix this:

The "Zero-Copy" part (Passing notes without rewriting them): In normal software, when a message moves from one part of the computer to another, the computer literally copies the entire message onto a new piece of paper. This wastes massive amounts of time and battery. My software just points to the original piece of paper and hands it over directly.

The "MLFQ" part (The VIP Emergency Line): If a routine temperature check is uploading to Earth, and suddenly a collision warning comes in, my software physically pauses the boring temperature check, shoves the emergency warning to the absolute front of the line, and sends it down to Earth immediately.

The "Rust" part (Titanium construction): Because I built it in Rust, it is virtually impossible for a random memory glitch or a radiation bit-flip in space to crash the whole system.

In simplest terms, I built the exact brain a satellite needs to decide what data to send home first, built in a way that uses almost zero electricity and never crashes.

 My project is an implementation of this entire system, using a real-world message queue (tokio::sync::mpsc), a hard-drive-backed log (MmapWal), and a CPU-efficient scheduler (PriorityScheduler) to simulate a real satellite.