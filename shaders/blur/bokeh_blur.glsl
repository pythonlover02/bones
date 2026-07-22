    {
        float kr = 4.0 * res_scale;
        vec2 d0 = vec2( 0.8535534,  0.3535534) * kr * inv;
        vec2 d1 = vec2(-0.3535534,  0.8535534) * kr * inv;
        vec2 d2 = vec2(-0.8535534, -0.3535534) * kr * inv;
        vec2 d3 = vec2( 0.3535534, -0.8535534) * kr * inv;
        vec3 k0a = texture(u_input, v_uv + d0).rgb;
        vec3 k0b = texture(u_input, v_uv - d0).rgb;
        vec3 k1a = texture(u_input, v_uv + d1).rgb;
        vec3 k1b = texture(u_input, v_uv - d1).rgb;
        vec3 k2a = texture(u_input, v_uv + d2).rgb;
        vec3 k2b = texture(u_input, v_uv - d2).rgb;
        vec3 k3a = texture(u_input, v_uv + d3).rgb;
        vec3 k3b = texture(u_input, v_uv - d3).rgb;
        float w0a = 1.0 + dot(k0a, ONE3) * 0.5;
        float w0b = 1.0 + dot(k0b, ONE3) * 0.5;
        float w1a = 1.0 + dot(k1a, ONE3) * 0.5;
        float w1b = 1.0 + dot(k1b, ONE3) * 0.5;
        float w2a = 1.0 + dot(k2a, ONE3) * 0.5;
        float w2b = 1.0 + dot(k2b, ONE3) * 0.5;
        float w3a = 1.0 + dot(k3a, ONE3) * 0.5;
        float w3b = 1.0 + dot(k3b, ONE3) * 0.5;
        vec3 ks = c + k0a*w0a + k0b*w0b + k1a*w1a + k1b*w1b
                    + k2a*w2a + k2b*w2b + k3a*w3a + k3b*w3b;
        float kn = 1.0 + w0a + w0b + w1a + w1b + w2a + w2b + w3a + w3b;
        c = ks / max(kn, 0.0001);
    }
