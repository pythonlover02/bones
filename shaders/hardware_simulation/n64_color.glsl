    {
        vec2 n64_d2 = v_uv - HALF2;
        c = mix(c, vec3(0.6, 0.65, 0.75), smoothstep(0.04, 0.5625, dot(n64_d2, n64_d2)) * 0.25);
        c = c * vec3(1.06, 1.02, 0.9);
        c = floor(c * 31.0 + 0.5 + bayer_signed(frag_coord) * 0.015) * 0.03225806;
    }
