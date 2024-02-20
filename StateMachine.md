# State Machines

## What is a state machine

- **Time** series sequence of operations
- **Other work** happens between steps
- **Waiting for** something else to finish before proceeding

## Requirements

- **Some value** that keeps track of current position in operation
- **Transition Logic** to move from one logical state to another
- **Other values** relevant to the current state

# Example

## Blinking light

```mermaid
graph LR;
    "Not Pressed" --> "Blink On";
    "Blink On" --> "Blink Off";
    "Blink Off" --> "Blink On";
    "Blink On" --> "Released Button";
    "Blink Off" --> "Released Button";
    "Released Button" --> "Not Pressed"
```

90% of software engineering is about managing State Machines