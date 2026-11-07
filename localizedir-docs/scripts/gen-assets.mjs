// Generates optimized logo derivatives from the repository's original asset
// (localizedir/docs/logo.png, 1264×1264 RGBA). Never upscales: the source is
// larger than every display size, so derivatives only downscale.
import sharp from "sharp";
import { mkdir } from "node:fs/promises";
import path from "node:path";

const SOURCE = "/home/z/my-project/localizedir/docs/logo.png";
const OUT = {
	logo: path.resolve("src/assets/logo.png"),
	favicon: path.resolve("public/favicon.png"),
};

await mkdir(path.dirname(OUT.logo), { recursive: true });
await mkdir(path.dirname(OUT.favicon), { recursive: true });

const source = sharp(SOURCE);
const meta = await source.metadata();
if ((meta.width ?? 0) < 256 || (meta.height ?? 0) < 256) {
	throw new Error(`source logo too small: ${meta.width}x${meta.height}`);
}

await sharp(SOURCE)
	.resize(256, 256, { fit: "inside", withoutEnlargement: true })
	.png({ palette: true, quality: 92, effort: 9 })
	.toFile(OUT.logo);

await sharp(SOURCE)
	.resize(64, 64, { fit: "inside", withoutEnlargement: true })
	.png({ palette: true, quality: 92, effort: 9 })
	.toFile(OUT.favicon);

for (const file of Object.values(OUT)) {
	const m = await sharp(file).metadata();
	console.log(file, `${m.width}x${m.height}`, m.format, "alpha:", m.hasAlpha);
}
