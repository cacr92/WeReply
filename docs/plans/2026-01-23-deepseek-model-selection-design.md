# DeepSeek Model Selection Design

## Goal
Fix DeepSeek connection failures by aligning request URLs with official docs, minimize request parameters to DeepSeek defaults, and add model selection that is automatically fetched after saving the API key and persisted across restarts.

## Scope
- Backend: correct API endpoints, add /models fetch, persist selected model in config.
- Frontend: fetch models after key save, allow selection, persist selection through backend.

## Non-Goals
- Expose full model or generation parameter configuration.
- Add additional providers or custom base URL UI.

## Architecture
The backend DeepSeek client builds URLs using the configured base URL plus endpoint paths (`/chat/completions`, `/models`) and sends minimal request payloads (model + messages). Model discovery uses `GET /models` with stored API key and filters to `deepseek-chat` and `deepseek-reasoner`. Configuration is persisted in `config.json` but only stores the selected model to keep settings minimal.

The frontend keeps a local `models` list and `selectedModel` state. After a successful API key save, it calls the new `list_models` command, normalizes the response to the two supported models, and updates the selection. When the user changes the selection, it calls `set_deepseek_model` to persist the choice. On bootstrap, it reads the persisted model via `get_config` and initializes the selector without re-fetching models.

## Error Handling
- Connection validation failures surface as existing error messages.
- Model list failures fall back to the two supported models with a warning message.
- Invalid model selection is rejected on the backend.

## Testing
- Unit tests for request payload minimality and URL building.
- Unit tests for model list normalization and selection resolution.
- Existing frontend and backend test suites remain green.
