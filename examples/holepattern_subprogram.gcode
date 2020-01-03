O0002

(T001 - 90-DEG SPOT DRILL)

N1 G21
N2 G17 G40 G80
N3 G90 G54 G00 X7.0 Y7.0 $1200 M03 T02
N4 G43 225.0 H01 M05
N5 G99 G82 R2.5 Z-3.4 P200 F200.0 L0 (OR K0 ON SOME CONTROLS)
N6 M98 P1001
N7 G80 G00 225.0 M09
N8 G28 225.0 M05
N9 M01

(TO2 - 5 MM TAP DRILL)
N10 T02
N11 M06
N12 G90 G54 G00 X7.0 Y7.0 S950 M03 T03
N13 G43 225.0 H02 M0B
N14 G99 G81 R2.5 Z-10.5 F300.0 L0 (OR K0 ON SOME CONTROLS)
N15 M98 P1001
N16 G80 G00 225.0 M09
N17 G28 225.0 M05
N18 M01

(T03 - M6X1 TAP)

N19 T03
N20 M06
N21 G90 G54 G00 X7.0 Y7.0 S600 M03 T01
N22 G43 225.0 H03 M0B
N23 G99 G84 R5.0 Z-11.0 F600.0 L0 (OR K0 ON SOME CONTROLS)
N24 M98 P1001
N25 G80 G00 225.0 M09
N26 G28 225.0 M05
N27 G28 X23.0 Y26.0
N28 M30

%

01001 (5 H0LE LOCATIONS SUBPROGRAM - VERSION 1)

N101 X7.0 Y7.0 (H1)
N102 X39.0 (H2)
N103 Y45.0 (H3)
N104 X7.0 (H4)
N105 X23.0 Y26.0 (H5)

N106 M99 (SUBPROGRAM END)
%
