struct VertexInput {
    @location(0) position: vec3<f32>, // <x, y, z>
    @location(1) color: vec4<f32>, // <r, g, b, a>
};

struct Data {
    sound_amplitude: f32,
    wind_strength: f32,
    noise_scale: f32,
    spring_constant: f32,
};

@group(0) @binding(0)
var<storage, read_write> vertices: array<VertexInput>;

@group(0) @binding(1)
var<storage, read> vertices_initial: array<VertexInput>;

@group(0) @binding(2)
var<uniform> data: Data;

@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;

    // Prevent out-of-bounds access
    if index >= arrayLength(&vertices) {
        return;
    }

    var current_position = vertices[index].position;

    // Simulate wind-like vector field using noise
    let wind = perlin_noise_3d(current_position * data.noise_scale) * data.wind_strength * data.sound_amplitude;

    // Apply wind force to the point's position
    current_position += wind;

    // Calculate the distance from the original position
    let displacement = current_position - vertices_initial[index].position;

    // Apply the spring-like restorative force
    current_position -= data.spring_constant * displacement;

    // Update the vertex position
    vertices[index].position = current_position;
}

fn permute4(x: vec4<f32>) -> vec4<f32> { return ((x * 34. + 1.) * x) % vec4<f32>(289.); }
fn taylorInvSqrt4(r: vec4<f32>) -> vec4<f32> { return 1.79284291400159 - 0.85373472095314 * r; }
fn fade3(t: vec3<f32>) -> vec3<f32> { return t * t * t * (t * (t * 6. - 15.) + 10.); }
fn fade4(t: vec4<f32>) -> vec4<f32> { return t * t * t * (t * (t * 6. - 15.) + 10.); }

fn perlin_noise_3d(P: vec3<f32>) -> f32 {
    var Pi0: vec3<f32> = floor(P); // Integer part for indexing
    var Pi1: vec3<f32> = Pi0 + vec3<f32>(1.); // Integer part + 1
    Pi0 = Pi0 % vec3<f32>(289.);
    Pi1 = Pi1 % vec3<f32>(289.);
    let Pf0 = fract(P); // Fractional part for interpolation
    let Pf1 = Pf0 - vec3<f32>(1.); // Fractional part - 1.
    let ix = vec4<f32>(Pi0.x, Pi1.x, Pi0.x, Pi1.x);
    let iy = vec4<f32>(Pi0.yy, Pi1.yy);
    let iz0 = Pi0.zzzz;
    let iz1 = Pi1.zzzz;

    let ixy = permute4(permute4(ix) + iy);
    let ixy0 = permute4(ixy + iz0);
    let ixy1 = permute4(ixy + iz1);

    var gx0: vec4<f32> = ixy0 / 7.;
    var gy0: vec4<f32> = fract(floor(gx0) / 7.) - 0.5;
    gx0 = fract(gx0);
    var gz0: vec4<f32> = vec4<f32>(0.5) - abs(gx0) - abs(gy0);
    var sz0: vec4<f32> = step(gz0, vec4<f32>(0.));
    gx0 = gx0 + sz0 * (step(vec4<f32>(0.), gx0) - 0.5);
    gy0 = gy0 + sz0 * (step(vec4<f32>(0.), gy0) - 0.5);

    var gx1: vec4<f32> = ixy1 / 7.;
    var gy1: vec4<f32> = fract(floor(gx1) / 7.) - 0.5;
    gx1 = fract(gx1);
    var gz1: vec4<f32> = vec4<f32>(0.5) - abs(gx1) - abs(gy1);
    var sz1: vec4<f32> = step(gz1, vec4<f32>(0.));
    gx1 = gx1 - sz1 * (step(vec4<f32>(0.), gx1) - 0.5);
    gy1 = gy1 - sz1 * (step(vec4<f32>(0.), gy1) - 0.5);

    var g000: vec3<f32> = vec3<f32>(gx0.x, gy0.x, gz0.x);
    var g100: vec3<f32> = vec3<f32>(gx0.y, gy0.y, gz0.y);
    var g010: vec3<f32> = vec3<f32>(gx0.z, gy0.z, gz0.z);
    var g110: vec3<f32> = vec3<f32>(gx0.w, gy0.w, gz0.w);
    var g001: vec3<f32> = vec3<f32>(gx1.x, gy1.x, gz1.x);
    var g101: vec3<f32> = vec3<f32>(gx1.y, gy1.y, gz1.y);
    var g011: vec3<f32> = vec3<f32>(gx1.z, gy1.z, gz1.z);
    var g111: vec3<f32> = vec3<f32>(gx1.w, gy1.w, gz1.w);

    let norm0 = taylorInvSqrt4(
        vec4<f32>(dot(g000, g000), dot(g010, g010), dot(g100, g100), dot(g110, g110))
    );
    g000 = g000 * norm0.x;
    g010 = g010 * norm0.y;
    g100 = g100 * norm0.z;
    g110 = g110 * norm0.w;
    let norm1 = taylorInvSqrt4(
        vec4<f32>(dot(g001, g001), dot(g011, g011), dot(g101, g101), dot(g111, g111))
    );
    g001 = g001 * norm1.x;
    g011 = g011 * norm1.y;
    g101 = g101 * norm1.z;
    g111 = g111 * norm1.w;

    let n000 = dot(g000, Pf0);
    let n100 = dot(g100, vec3<f32>(Pf1.x, Pf0.yz));
    let n010 = dot(g010, vec3<f32>(Pf0.x, Pf1.y, Pf0.z));
    let n110 = dot(g110, vec3<f32>(Pf1.xy, Pf0.z));
    let n001 = dot(g001, vec3<f32>(Pf0.xy, Pf1.z));
    let n101 = dot(g101, vec3<f32>(Pf1.x, Pf0.y, Pf1.z));
    let n011 = dot(g011, vec3<f32>(Pf0.x, Pf1.yz));
    let n111 = dot(g111, Pf1);

    var fade_xyz: vec3<f32> = fade3(Pf0);
    let temp = vec4<f32>(f32(fade_xyz.z)); // simplify after chrome bug fix
    let n_z = mix(vec4<f32>(n000, n100, n010, n110), vec4<f32>(n001, n101, n011, n111), temp);
    let n_yz = mix(n_z.xy, n_z.zw, vec2<f32>(f32(fade_xyz.y))); // simplify after chrome bug fix
    let n_xyz = mix(n_yz.x, n_yz.y, fade_xyz.x);
    return 2.2 * n_xyz;
}
