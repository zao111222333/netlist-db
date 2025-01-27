* aa
.inc 'inc/inc1.sp'
X0.AAA net8 net23 VSS VPW NHVT11LL_CKT W=0.135u L=40.00n
.subckt DEMO A1 A2 var1=1 var2=2
.inc 'inc/inc0.sp'
.inc 'inc/inc2.sp'
.ends
.lib 'lib.sp' tt

.subckt UNITS A
.inc 'units.sp'
.ends