    {
        float br = res_scale;
        vec2 hp = vec2(br * 0.5 + 0.5) * inv;
        vec3 k0 = texture(u_input, v_uv + vec2( hp.x,  hp.y)).rgb;
        vec3 k1 = texture(u_input, v_uv + vec2( hp.x, -hp.y)).rgb;
        vec3 k2 = texture(u_input, v_uv + vec2(-hp.x,  hp.y)).rgb;
        vec3 k3 = texture(u_input, v_uv + vec2(-hp.x, -hp.y)).rgb;
        vec2 hp2 = vec2(br * 1.5 + 0.5) * inv;
        vec3 k4 = texture(u_input, v_uv + vec2( hp2.x,  hp2.y)).rgb;
        vec3 k5 = texture(u_input, v_uv + vec2( hp2.x, -hp2.y)).rgb;
        vec3 k6 = texture(u_input, v_uv + vec2(-hp2.x,  hp2.y)).rgb;
        vec3 k7 = texture(u_input, v_uv + vec2(-hp2.x, -hp2.y)).rgb;
        c = (c + k0 + k1 + k2 + k3 + k4 + k5 + k6 + k7) * 0.11111111;
    }
