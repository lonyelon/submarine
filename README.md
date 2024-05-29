# Submarine

An underwater drone made for exploration at "Lagoa de Pedras Miúdas".

## System design

This repository has three independent components:

| Component | Description |
| --------- | ----------- |
| land      | Software run on the main (land) computer. Shows the camera feed from the onboard computer and sends commands to it. |
| onboard   | Software that runs on the onboard computer. Relays the camera feed (and other data) to the land computer and relays it's commands to the microcontroller. |
| micro     | Software that runs on the onboard microcontroller. Receives orders from the onboard computer and controls all systems accordingly. It also receives data from all sensors (except the camera) and sends it to the onboard computer. |
