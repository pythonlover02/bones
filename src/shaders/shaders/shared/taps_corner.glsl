    vec3 tap_1_1   = texture(u_input, v_uv + vec2( inv.x,  inv.y)).rgb;
    vec3 tap_m1_1  = texture(u_input, v_uv + vec2(-inv.x,  inv.y)).rgb;
    vec3 tap_1_m1  = texture(u_input, v_uv + vec2( inv.x, -inv.y)).rgb;
    vec3 tap_m1_m1 = texture(u_input, v_uv + vec2(-inv.x, -inv.y)).rgb;
