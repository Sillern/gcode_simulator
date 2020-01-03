%  Signals start of data during file transfer. Originally used to stop tape rewind, not necessarily start of program. For some controls (FANUC) the first LF (EOB) is start of program. ISO uses %, EIA uses ER (0x0B).
O4968 (OPTIONAL PROGRAM DESCRIPTION OR COMMENT) ; Sample face and turn program. Comments are enclosed in parentheses.
N01 M216 ; Turn on load monitor
N02 G20 G90 G54 D200 G40 ; Inch units. Absolute mode. Activate work offset. Activate tool offset. Deactivate tool nose radius compensation.
N03 G50 S2000 ; Set maximum spindle speed in rev/min — This setting affects Constant Surface Speed mode
N04 T0300 ;Index turret to tool 3. Clear wear offset (00).
N05 G96 S854 M03 ;Constant surface speed [automatically varies the spindle speed], 854 sfm, start spindle CW rotation
N06 G41 G00 X1.1 Z1.1 T0303 M08 ;Enable cutter radius compensation mode, rapid position to 0.55" above axial centerline (1.1" in diameter) and 1.1 inches positive from the work offset in Z, activate flood coolant
N07 G01 Z1.0 F.05 ;Feed in horizontally at rate of 0.050" per revolution of the spindle until the tool is positioned 1" positive from the work offset
N08 X-0.016 ;Feed the tool slightly past center—the tool must travel by at least its nose radius past the center of the part to prevent a leftover scallop of material.
N09 G00 Z1.1; Rapid positioning; retract to start position
N10 X1.0 Rapid positioning; next pass
N11 G01 Z0.0 F.05 ;Feed in horizontally cutting the bar to 1" diameter all the way to the datum, 0.05in/rev
N12 G00 X1.1 M05 M09 ;Clear the part, stop the spindle, turn off the coolant
N13 G91 G28 X0 ;Home X axis — return the machine's home position for the X axis
N14 G91 G28 Z0; Home Z axis — return to machine's home position for the Z axis
N15 G90 ;Return to absolute mode. Turn off load monitor
N16 M30; Program stop, rewind to top of program, wait for cycle start
%  Signal end of data during file transfer. Originally used to mark end of tape, not necessarily end of program. ISO uses %, EIA uses ER (0x0B).
