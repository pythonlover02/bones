    {
        vec3 ds = vec3(dot(c, vec3(0.625, 0.375, 0.0)),
                       dot(c, vec3(0.7, 0.3, 0.0)),
                       dot(c, vec3(0.0, 0.3, 0.7)));
        vec3 de = c - ds;
        c = clamp(c + vec3(0.0, de.r * 0.7, de.r * 0.7), 0.0, 1.0);
    }
