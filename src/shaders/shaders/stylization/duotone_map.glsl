    {
        float dl = dot(c, LUMA_BT601);
        c = mix(vec3(0.1, 0.1, 0.3), vec3(1.0, 0.9, 0.7), dl);
    }
