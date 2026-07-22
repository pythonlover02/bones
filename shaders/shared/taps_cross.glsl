    vec3 tap_1_0  = texture(u_input, v_uv + vec2( inv.x, 0.0)).rgb;
    vec3 tap_m1_0 = texture(u_input, v_uv + vec2(-inv.x, 0.0)).rgb;
    vec3 tap_0_1  = texture(u_input, v_uv + vec2(0.0,  inv.y)).rgb;
    vec3 tap_0_m1 = texture(u_input, v_uv + vec2(0.0, -inv.y)).rgb;
