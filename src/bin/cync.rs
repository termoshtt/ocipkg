use oci_spec::image::{
    Descriptor, DescriptorBuilder, ImageManifestBuilder, MediaType, SCHEMA_VERSION,
};

fn main() {
    let config = DescriptorBuilder::default()
        .media_type(MediaType::ImageConfig)
        .size(7023)
        .digest("sha256:b5b2b2c507a0944348e0303114d8d93aaaa081732b86451d9bce1f432a537bc7")
        .build()
        .expect("build config descriptor");

    let layers: Vec<Descriptor> = [
        (
            32654,
            "sha256:9834876dcfb05cb167a5c24953eba58c4ac89b1adf57f28f2f9d09af107ee8f0",
        ),
        (
            16724,
            "sha256:3c3a4604a545cdc127456d94e421cd355bca5b528f4a9c1905b15da2eb4a4c6b",
        ),
        (
            73109,
            "sha256:ec4b8955958665577945c89419d1af06b5f7636b4ac3da7f12184802ad867736",
        ),
    ]
    .iter()
    .map(|l| {
        DescriptorBuilder::default()
            .media_type(MediaType::ImageLayerGzip)
            .size(l.0)
            .digest(l.1.to_owned())
            .build()
            .expect("build layer")
    })
    .collect();

    let image_manifest = ImageManifestBuilder::default()
        .schema_version(SCHEMA_VERSION)
        .config(config)
        .layers(layers)
        .build()
        .expect("build image manifest");

    image_manifest
        .to_writer_pretty(&mut std::io::stdout())
        .unwrap();
}
