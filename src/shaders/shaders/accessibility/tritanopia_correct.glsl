    {
        vec3 ds = vec3(dot(c, vec3(0.95, 0.05, 0.0)),
                       dot(c, vec3(0.0, 0.433, 0.567)),
                       dot(c, vec3(0.0, 0.475, 0.525)));
        vec3 de = c - ds;
        c = clamp(c + vec3(de.b * 0.7, de.b * 0.7, 0.0), 0.0, 1.0);
    }
