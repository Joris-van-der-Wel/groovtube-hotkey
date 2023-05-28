# GroovTube Hotkey
Cross-platform program for translating human breath input to mouse and keyboard hotkeys using the GroovTube BLE peripheral.

The GroovTube is a unique device which makes breathing and oral motor skills visible and analyzable in a fun way. It turns your breath into real-time image effects, for people with breathing, speech, or oral motor disabilities. For more details see: https://www.groovtube.nl/en

This program will automatically search for a GroovTube peripheral using Bluetooth Low Energy and connect to it. It will then continuously receive breath strength values, which is a percentage from 0% to 100% of sip or puff strength. Using a Graphical User Interface, the user may define thresholds at which a hotkey will trigger. For example the user could decide that the program should hold down the left mouse button, if the puff strength is over 20%.
