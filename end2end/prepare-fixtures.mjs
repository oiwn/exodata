import { copyFile, mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const end2endDir = dirname(fileURLToPath(import.meta.url));
const repositoryDir = dirname(end2endDir);
const sourceDir = join(repositoryDir, "crates", "exo-core", "tests", "fixtures");
const outputDir = join(end2endDir, "runtime-data");

await mkdir(outputDir, { recursive: true });

for (const dataset of ["stellarhosts", "exoplanets"]) {
  const manifest = JSON.parse(
    await readFile(join(sourceDir, `${dataset}.fixture.json`), "utf8"),
  );

  if (manifest.column_names.length !== manifest.dtypes.length) {
    throw new Error(`${dataset} fixture schema is inconsistent`);
  }

  await copyFile(
    join(sourceDir, `${dataset}.fixture`),
    join(outputDir, `${dataset}.parquet`),
  );

  const metadata = manifest.column_names
    .map(
      (name, index) =>
        `[[column]]\nname = ${JSON.stringify(name)}\ndatatype = ${JSON.stringify(manifest.dtypes[index])}\n`,
    )
    .join("\n");

  await writeFile(
    join(outputDir, `${dataset}-metadata.toml`),
    metadata,
    "utf8",
  );
}
