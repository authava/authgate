export interface ScopeRequirement {
  resource_type: string
  action: string
  resource_id?: string
}

export interface TeamRequirement {
  id?: string
  name?: string
  scopes?: ScopeRequirement[]
}

export interface RequireConfig {
  roles?: string[]
  permissions?: string[]
  scopes?: ScopeRequirement[]
  teams?: TeamRequirement[]
}

export interface Route {
  id?: string
  host: string
  path: string
  require: RequireConfig
}

export interface ApiError {
  status: string
  message: string
}

export interface ApiResponse<T> {
  data?: T
  error?: ApiError
}
