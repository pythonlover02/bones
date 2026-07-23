    vec3 c;
    {
        vec2 ca_d = (v_uv - HALF2) * 0.005;
        c.r = texture(u_input, v_uv + ca_d).r;
        c.g = texture(u_input, v_uv).g;
        c.b = texture(u_input, v_uv - ca_d).b;
    }
