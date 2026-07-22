    {
        float vhs_g = dot(c, LUMA_BT601);
        c = mix(vec3(vhs_g), c, 0.7);
        c = c * vec3(1.04, 1.0, 0.88);
    }
