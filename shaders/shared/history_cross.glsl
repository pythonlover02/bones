    vec3 hist_e = texture(u_history, v_uv + vec2( inv.x, 0.0)).rgb;
    vec3 hist_w = texture(u_history, v_uv + vec2(-inv.x, 0.0)).rgb;
    vec3 hist_n = texture(u_history, v_uv + vec2(0.0,  inv.y)).rgb;
    vec3 hist_s = texture(u_history, v_uv + vec2(0.0, -inv.y)).rgb;
