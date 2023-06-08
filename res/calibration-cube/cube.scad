// cube side length
side=20;
// font size percentage
ptext=side*0.8;
// letters depth
depth=2;

cube_xyz(side, ptext, depth);

module cube_xyz(i_side,i_ptext,i_depth) {
	insurance=0.1;
	
	difference() {
		cube(i_side, false);
		
        
		translate([i_side - i_depth, i_side / 2, i_side / 2]) {
            rotate(a=[90, 0, 90]) {
                linear_extrude(i_depth + insurance) {
                    text("X", size = i_ptext, halign = "center", valign = "center");
                }
            }
        }
    
		translate([i_side / 2, i_side + i_depth, i_side / 2]) {
            rotate(a=[90, 90, 0]) {
                linear_extrude(2*i_depth) {
                    text("Y", size = i_ptext, halign = "center", valign = "center");
                }
            }
        }
		
		translate([i_side/2,i_side/2,i_side - i_depth])
        linear_extrude(i_depth+insurance)
        text("Z",size=i_ptext,halign="center",valign="center");
	}
}
