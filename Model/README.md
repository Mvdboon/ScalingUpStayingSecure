# Intro
This library was created in the process of doing a Master's Thesis in Computer Science at the TU Delft in the specialization of Cyber Security.
It was created by Mike van der Boon. You can reach out to him by emailing <thesis_library@mvdboon.nl>. 

This library simulates the power grid in various stages of its transformation towards the "Smart Grid". It aims to find the cyber-related risks that are related to this.
Within these risks, it focuses on IoT-related risks, risks that are the result of vulnerabilities in the firmware of the Distributed Energy Resources (DER).

The simulation is an [Agent-Based Model (ABM)](https://en.wikipedia.org/wiki/Agent-based_model), that determines the system behaviour by the results of autonomous agents.

For more information see the thesis at [link](https://repository.tudelft.nl/islandora/object/uuid:0e0bb7e2-0ca8-4044-aadb-27bc200cebf8).

# Model Layout
![Layout of the model][layoutmodel]

# Actions that are taken per step
1) Update the inner step variable of the agents to the new step.
2) Calculate the "clean" powerstate for each agent that can generate power.
3) Try to patch and infect the relevant agents.
4) Change the powerstate of the infected agents.
5) Calculate the powerstate of agents that don't generate power by combining the powerstates of its children.
6) Try to compensate for the power mismatch on the grid level using regulating margin.
7) Determine the effects of the power mismatch on the frequency of the grid and the voltage at the netstation level.
8) Check if the current state is outside of normal operating limits.
9) Update the history states of the agents.
10) Output the state of the model if desired.
