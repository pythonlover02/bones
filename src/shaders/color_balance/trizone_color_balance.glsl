    {
        float cl = dot(c, LUMA_BT601);
        float cs = 1.0 - smoothstep(0.0, 0.4, cl);
        float ch = smoothstep(0.6, 1.0, cl);
        c = c + vec3(-0.03, 0.01, 0.02) * cs + vec3(0.02, 0.0, -0.02) * ch;
    }
