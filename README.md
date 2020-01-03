Simple GCode simulator
================

Aims to simulate a machine handling GCode.

Test with:
`cargo run examples/holepattern.gcode`


GCode command words
----------------

Information from wikipedia:
* A
  * Absolute or incremental position of A axis (rotational axis around X axis)
  * Positive rotation is defined as a counterclockwise rotation looking from X positive towards X negative.
* B
  * Absolute or incremental position of B axis (rotational axis around Y axis)
* C
  * Absolute or incremental position of C axis (rotational axis around Z axis)
* U
  * Incremental axis corresponding to X axis
* V
  * Incremental axis corresponding to Y axis 
* W
  * Incremental axis corresponding to Z axis 
* X
  * Absolute or incremental position of X axis.
* Y
  * Absolute or incremental position of Y axis
* Z
  * Absolute or incremental position of Z axis 

* G
  * Address for preparatory commands G commands often tell the control what kind of motion is wanted (e.g., rapid positioning, linear feed, circular feed, fixed cycle) or what offset value to use.
* M
  * Miscellaneous function Action code, auxiliary command; descriptions vary. Many M-codes call for machine functions, which is why people often say that the "M" stands for "machine", although it was not intended to.

* D
  * Defines diameter or radial offset used for cutter compensation. D is used for depth of cut on lathes. It is used for aperture selection and commands on photoplotters. G41: left cutter compensation, G42: right cutter compensation
* E
  * Precision feedrate for threading on lathes
* F
  * Defines feed rate Common units are distance per time for mills (inches per minute, IPM, or millimeters per minute, mm/min) and distance per revolution for lathes (inches per revolution, IPR, or millimeters per revolution, mm/rev)
* H
  * Defines tool length offset;
  * Incremental axis corresponding to C axis (e.g., on a turn-mill) G43: Negative tool length compensation, G44: Positive tool length compensation
* I   
  * Defines arc center in X axis for G02 or G03 arc commands.
  * Also used as a parameter within some fixed cycles. The arc center is the relative distance from the current position to the arc center, not the absolute distance from the work coordinate system (WCS).
* J
  * Defines arc center in Y axis for G02 or G03 arc commands.
  * Also used as a parameter within some fixed cycles. Same corollary info as I above.
* K
  * Defines arc center in Z axis for G02 or G03 arc commands.
  * Also used as a parameter within some fixed cycles, equal to L address. Same corollary info as I above.
* L
  * Fixed cycle loop count;
  * Specification of what register to edit using G10 Fixed cycle loop count: Defines number of repetitions ("loops") of a fixed cycle at each position. Assumed to be 1 unless programmed with another integer. Sometimes the K address is used instead of L. With incremental positioning (G91), a series of equally spaced holes can be programmed as a loop rather than as individual positions.
  * G10 use: Specification of what register to edit (work offsets, tool radius offsets, tool length offsets, etc.).
* N
  * Line (block) number in program;
  * System parameter number to change using G10 Line (block) numbers: Optional, so often omitted. Necessary for certain tasks, such as M99 P address (to tell the control which block of the program to return to if not the default) or GoTo statements (if the control supports those). N numbering need not increment by 1 (for example, it can increment by 10, 20, or 1000) and can be used on every block or only in certain spots throughout a program.
  * System parameter number: G10 allows changing of system parameters under program control.[8]
* O
  * Program name For example, O4501. 
* P
  * Serves as parameter address for various G and M codes
  * With G04, defines dwell time value.
  * Also serves as a parameter in some canned cycles, representing dwell times or other variables.
  * Also used in the calling and termination of subprograms. (With M98, it specifies which subprogram to call; with M99, it specifies which block number of the main program to return to.)
* Q
  * Peck increment in canned cycles For example, G73, G83 (peck drilling cycles)
* R
  * Defines size of arc radius, or defines retract height in milling canned cycles For radii, not all controls support the R address for G02 and G03, in which case IJK vectors are used. For retract height, the "R level", as it's called, is returned to if G99 is programmed.
* S
  * Defines speed, either spindle speed or surface speed depending on mode Data type = integer. In G97 mode (which is usually the default), an integer after S is interpreted as a number of rev/min (rpm). In G96 mode (Constant Surface Speed or CSS), an integer after S is interpreted as surface speed—sfm (G20) or m/min (G21). See also Speeds and feeds. On multifunction (turn-mill or mill-turn) machines, which spindle gets the input (main spindle or subspindles) is determined by other M codes.
* T
  * Tool selection To understand how the T address works and how it interacts (or not) with M06, one must study the various methods, such as lathe turret programming, ATC (Automatic Tool Change, set by M06) fixed tool selection, ATC random memory tool selection, the concept of "next tool waiting", and empty tools.[5] Programming on any particular machine tool requires knowing which method that machine uses.[5]
