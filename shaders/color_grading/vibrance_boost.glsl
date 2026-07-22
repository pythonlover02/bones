    {
        float vmx = max(c.r, max(c.g, c.b));
        float vmn = min(c.r, min(c.g, c.b));
        float vl = dot(c, LUMA_BT601);
        c = mix(vec3(vl), c, 1.0 + 0.5 * (1.0 - (vmx - vmn)));
    }
