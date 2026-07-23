    vec3 grad_x_rgb = tap_1_0 - tap_m1_0;
    vec3 grad_y_rgb = tap_0_1 - tap_0_m1;
    float lgrad_x = dot(grad_x_rgb, LUMA_AVG);
    float lgrad_y = dot(grad_y_rgb, LUMA_AVG);
