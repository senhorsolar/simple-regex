Simple regex engine implementation:
1. Parses regex pattern using simple recursive descent
2. Converts regex pattern to NFA
3. Simulates NFA using algorithm from dragon book section 3.7.2

Usage:
`cargo run -- <pattern> <input>`
