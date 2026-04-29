# az sig (Shared Image Gallery)

Read-only commands for Shared Image Gallery (`Microsoft.Compute/galleries`), image definitions, and image versions. Includes shared and community gallery lookups.

Create / update / delete operations are intentionally not implemented in this pass.

## Commands

### Top-level

| Command | Description |
|---|---|
| `sig list [-g <rg>]` | List galleries in subscription or resource group |
| `sig show -g <rg> -r <gallery>` | Show a gallery |
| `sig list-shared -l <loc> [--shared-to tenant]` | List galleries shared to your subscription (or tenant) |
| `sig show-shared -l <loc> -r <galleryUniqueName>` | Show a shared gallery |
| `sig list-community -l <loc>` | List community galleries (via Resource Graph, like `az`) |
| `sig show-community -l <loc> -p <publicGalleryName>` | Show a community gallery |

### `sig image-definition`

| Command | Description |
|---|---|
| `sig image-definition list -g <rg> -r <gallery>` | List image definitions in a gallery |
| `sig image-definition show -g <rg> -r <gallery> -i <def>` | Show one image definition |
| `sig image-definition list-shared -l <loc> -r <galleryUniqueName>` | List image definitions in a shared gallery |
| `sig image-definition show-shared -l <loc> -r <galleryUniqueName> -i <def>` | Show a shared image definition |
| `sig image-definition list-community -l <loc> -p <publicGalleryName>` | List image definitions in a community gallery |
| `sig image-definition show-community -l <loc> -p <publicGalleryName> -i <def>` | Show a community image definition |

### `sig image-version`

| Command | Description |
|---|---|
| `sig image-version list -g <rg> -r <gallery> -i <def>` | List image versions |
| `sig image-version show -g <rg> -r <gallery> -i <def> -e <version>` | Show one image version |
| `sig image-version list-shared -l <loc> -r <galleryUniqueName> -i <def>` | List versions in a shared gallery image |
| `sig image-version show-shared -l <loc> -r <galleryUniqueName> -i <def> -e <version>` | Show a shared image version |
| `sig image-version list-community -l <loc> -p <publicGalleryName> -i <def>` | List versions in a community gallery image |
| `sig image-version show-community -l <loc> -p <publicGalleryName> -i <def> -e <version>` | Show a community image version |

## Examples

```bash
# Galleries
azcli sig list -o table
azcli sig list -g my-rg -o table
azcli sig show -g my-rg -r my-gallery

# Image definitions in a gallery
azcli sig image-definition list -g my-rg -r my-gallery -o table
azcli sig image-definition show -g my-rg -r my-gallery -i my-image -o table

# Image versions
azcli sig image-version list -g my-rg -r my-gallery -i my-image -o table
azcli sig image-version show -g my-rg -r my-gallery -i my-image -e 1.0.0

# Community galleries (e.g. AKS public images)
azcli sig list-community -l uksouth -o table
azcli sig show-community -l uksouth -p AKSUbuntu-<unique-id>
azcli sig image-definition list-community -l uksouth -p AKSUbuntu-<unique-id> -o table
azcli sig image-version list-community -l uksouth -p AKSUbuntu-<unique-id> -i 1804gen2gpucontainerd -o table
```

## API versions

| Resource provider | API version |
|---|---|
| `Microsoft.Compute/galleries` (+ `images`, `versions`) | `2024-03-03` |
| `Microsoft.Compute/locations/sharedGalleries` (+ images, versions) | `2024-03-03` |
| `Microsoft.Compute/locations/communityGalleries` (+ images, versions) | `2024-03-03` |
| `Microsoft.ResourceGraph/resources` (used by `sig list-community`) | `2021-03-01` |

## Notes

- `sig list-community` has no native ARM list endpoint; both `az` and `azcli` query Azure Resource Graph (`communitygalleryresources` table). Limit defaults to 30.
- Short flags follow `az` conventions: `-r` = gallery name / unique name, `-i` = image definition, `-e` = image version, `-p` = public gallery name.
- `--shared-to tenant` on `sig list-shared` queries tenant-scoped shares; default is subscription-scoped.
