O0001

(TO1 - 90-DEG SPOT DRILL)
N1 G21
N2 G17 G40 G80
N3 G90 G54 G00 X7.0 Y7.0 S1200 M03 T02 (H1)
N4 G43 Z25.0 H01 M08
N5 G99 G82 R2.5 Z-3.4 P200 F200.0
N6 X39.0 (H2)
N7 Y45.0 (H3)
N8 X7.0 (H4)
N9 X23.0 Y26.0 (H5)
N10 G80 G00 225.0 M09
N11 G28 225.0 M05
N12 M01

(TO2 - 5 MM TAP DRILL)
N13 T02
N14 M06
N15 G90 G54 G00 X7.0 Y7.0 S950 M03 T03 (H1)
N16 G43 Z25.0 H02 M08
N17 G99 G81 R2.5 Z-10.5 F300.0
N18 X39.0 (H2)
N19 Y45.0 (H3)
N20 X7.0 (H4)
N21 X23.0 Y26.0 (H5)
N22 G80 G00 225.0 M09
N23 G28 225.0 M05
N24 M01

(TO3 - M6X1 TAP)
N25 T03
N26 M06
N27 G90 G54 G00 X7.0 Y7.0 S600 M03 T01 (H1)
N28 G43 Z25.0 H03 M08
N29 G99 G84 R5.0 Z-11.0 F600.0
N30 X39.0 (H2)
N31 Y45.0 (H3)
N32 X7.0 (H4)
N33 X23.0 Y26.0 (H5)
N34 G80 G00 225.0 M09
N35 G28 225.0 M05
N36 G28 X23.0 Y26.0
N37 M30

%