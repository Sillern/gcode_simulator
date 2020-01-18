O0003

(EXAMPLE 3 OF 3 - MAIN PROGRAM WITH A SUBPROGRAM - PETER SMID)
(PROGRAM ZERO IS AT L0WER LEFT CORNER AND T0P OF PART)

(T01 - 90-DEG SPOT DRILL)
N1 G21
N2 G17 G40 G80
N3 G90 G54 G00 X7.0 Y7.0 S1200 M03 T02 (#1)
N4 G43 Z25.0 H01 M08
NS G99 G82 R2.5 Z-3.4 P200 F200.0 L0 (OR K0 ON SOME CONTROLS)
N6 M98 P1002
N7 M01

(T02 - 5 MM TAP DRILL)
N8 T02
N9 M06
N10 G90 G54 G00 X7.0 Y7.0 S950 M03 T03 (H1)
N1l G43 Z25.0 H02 M05
N12 G99 G81 R2.5 Z-10.5 F300.0 L0 (OR K0 ON SOME CONTROLS)
N13 M98 P1002
N14 M01

(T03 - M6X1 TAP)
N15 T03
N16 M06
N17 G90 G54 G00 X7.0 Y7.0 S600 M03 T01 (1)
N18 G43 Z25.0 H03 M05
N19 G99 G84 R5.0 Z-11.0 F600.0 L0 (OR K0 ON SOME CONTROLS)
N20 M98 P1002
N21 G28 X23.0 Y26.0
N22 M30


O1002 (5 H0LE L0CATIONS SUBPROGRAM - VERSION 2)
N101 X7.0 Y7.0 (H1)
N102 X39.0 (H2)
N103 Y45.0 (H3)
N104 X7.0 (H4)
N105 X23.0 Y26.0 (H5)
N106 G80 G00 Z25.0 M05 (CANCEL CYCLE AND CLEAR)
N107 G28 Z25.0 M05 (Z-AXIS H0ME RETURN)
N108 M99  (SUBPROGRAM END)

%
