# ALICE-Medical-SaaS

Medical imaging processing as a Service — DICOM ingestion, volumetric segmentation, 3D reconstruction, and windowing via REST API.

## Architecture

```
Client
  |
  v
API Gateway (:8218)
  |
  v
Core Engine (:8118)
  |
  +-- DICOM Parser
  +-- Segmentation Engine
  +-- Volume Reconstructor
  +-- Windowing Processor
```

## Features

- DICOM series ingestion and metadata extraction
- Multi-organ semantic segmentation (CT, MRI, PET)
- Volumetric 3D mesh reconstruction (marching cubes)
- Hounsfield unit windowing presets (bone, lung, brain, liver)
- PACS-compatible output (NIfTI, NRRD, STL)
- Anonymization pipeline (HIPAA-ready)

## API Endpoints

### Core Engine (port 8118)

| Method | Path | Description |
|--------|------|-------------|
| POST | /api/v1/medical/segment | Segment anatomical structures from a volume |
| POST | /api/v1/medical/reconstruct | Generate 3D mesh from segmentation mask |
| POST | /api/v1/medical/windowing | Apply HU windowing to a DICOM series |
| POST | /api/v1/medical/dicom | Ingest and parse a DICOM series |
| GET  | /api/v1/medical/stats | Return runtime statistics |
| GET  | /health | Health check |

### Example: DICOM Ingest

```bash
curl -X POST http://localhost:8118/api/v1/medical/dicom \
  -H 'Content-Type: application/json' \
  -d '{"series_uid":"1.2.3.4","modality":"CT","slice_count":256}'
```

### Example: Segmentation

```bash
curl -X POST http://localhost:8118/api/v1/medical/segment \
  -H 'Content-Type: application/json' \
  -d '{"series_uid":"1.2.3.4","targets":["liver","spleen"],"model":"totalsegmentator-v2"}'
```

## License

AGPL-3.0-or-later
