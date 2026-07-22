    {
        const vec3 ANA_THR = vec3(0.7);
        float ax_unit = 6.0 * res_scale * inv.x;
        float as_x  = ax_unit * 1.5;
        float as_x2 = ax_unit * 3.5;
        float as_x3 = ax_unit * 5.5;
        vec3 a1p = texture(u_input, v_uv + vec2( as_x, 0.0)).rgb;
        vec3 a1m = texture(u_input, v_uv + vec2(-as_x, 0.0)).rgb;
        vec3 a2p = texture(u_input, v_uv + vec2( as_x2, 0.0)).rgb;
        vec3 a2m = texture(u_input, v_uv + vec2(-as_x2, 0.0)).rgb;
        vec3 a3p = texture(u_input, v_uv + vec2( as_x3, 0.0)).rgb;
        vec3 a3m = texture(u_input, v_uv + vec2(-as_x3, 0.0)).rgb;
        vec3 as2 = (max(a1p - ANA_THR, ZERO3) + max(a1m - ANA_THR, ZERO3)) * 1.5
                 + (max(a2p - ANA_THR, ZERO3) + max(a2m - ANA_THR, ZERO3)) * 0.58333333
                 + (max(a3p - ANA_THR, ZERO3) + max(a3m - ANA_THR, ZERO3)) * 0.36666667;
        c = c + as2 * 0.3 * vec3(0.4, 0.5, 1.0);
    }
