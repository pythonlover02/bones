    {
        float sl = dot(c, LUMA_BT601);
        c = c + mix(vec3(-0.1, 0.0, 0.1), vec3(0.1, 0.0, -0.1), smoothstep(0.0, 1.0, sl)) * 0.3;
    }
