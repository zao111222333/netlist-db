.ic v(net61) = 2.52678000e-05
.ic v(Y) = 7.00000000e-01

*** parameter section
.param A_t0=A_t_settle+t_settle_shift
.param A_t1=A_t0+A_t_slew
.param A_t_settle=0.00000000e+00
.param A_t_slew=4.00000000e-07
.param A_v0=7.00000000e-01
.param A_v1=0.00000000e+00
.param B_v=7.00000000e-01
.param C_v=7.00000000e-01
.param Y_cap=4.60800000e-11
.param t_settle_shift=0.00000000e+00
.param time_duration=1.00000000e-07
.param tran_step=1.00000000e-12
.param tran_time_end=time_duration+t_settle_shift
* sweep data parameter

*** print section

*** voltage section
VA A 0 pwl(
+ A_t0 A_v0
+ A_t1 A_v1)
VB B 0 B_v
VC C 0 C_v
VGND GND 0 0.00000000e+00
VVDD VDD 0 7.00000000e-01
VVSS VSS 0 0.00000000e+00

*** cap section
CY_cap_0 Y 0 Y_cap

*** r section

*** measure section
.meas tran BTDCell_TransitionBgn_Y TRIG at=0.00000000e+00 TARG v(Y) val=4.90000000e-01 fall=last
.meas tran BTDCell_TransitionEnd_Y TRIG at=0.00000000e+00 TARG v(Y) val=2.10000000e-01 fall=last
