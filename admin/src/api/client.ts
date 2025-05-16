import type { ApiError, ApiResponse, Route } from '../types'

// Get the API URL from environment variables
const API_URL = import.meta.env.VITE_API_URL || '/admin'

// Common fetch options to include credentials
const fetchOptions: RequestInit = {
  credentials: 'include',
  headers: {
    'Content-Type': 'application/json',
  },
}

// Helper function to handle API responses
async function handleResponse<T>(response: Response): Promise<ApiResponse<T>> {
  if (!response.ok) {
    const errorData: ApiError = await response.json().catch(() => ({
      status: 'error',
      message: `HTTP error ${response.status}: ${response.statusText}`,
    }))

    return { error: errorData }
  }

  const data: T = await response.json()
  return { data }
}

// API client functions
export const api = {
  // Get all routes
  async getRoutes(): Promise<ApiResponse<Route[]>> {
    try {
      const response = await fetch(`${API_URL}/routes`, fetchOptions)
      return handleResponse<Route[]>(response)
    } catch (error) {
      return {
        error: {
          status: 'error',
          message: `Failed to fetch routes: ${error instanceof Error ? error.message : String(error)}`,
        },
      }
    }
  },

  // Get a single route by ID
  async getRoute(id: string): Promise<ApiResponse<Route>> {
    try {
      const response = await fetch(`${API_URL}/routes/${id}`, fetchOptions)
      return handleResponse<Route>(response)
    } catch (error) {
      return {
        error: {
          status: 'error',
          message: `Failed to fetch route: ${error instanceof Error ? error.message : String(error)}`,
        },
      }
    }
  },

  // Create a new route
  async createRoute(route: Route): Promise<ApiResponse<Route>> {
    try {
      const response = await fetch(`${API_URL}/routes`, {
        ...fetchOptions,
        method: 'POST',
        body: JSON.stringify(route),
      })
      return handleResponse<Route>(response)
    } catch (error) {
      return {
        error: {
          status: 'error',
          message: `Failed to create route: ${error instanceof Error ? error.message : String(error)}`,
        },
      }
    }
  },

  // Update an existing route
  async updateRoute(id: string, route: Route): Promise<ApiResponse<Route>> {
    try {
      const response = await fetch(`${API_URL}/routes/${id}`, {
        ...fetchOptions,
        method: 'PUT',
        body: JSON.stringify(route),
      })
      return handleResponse<Route>(response)
    } catch (error) {
      return {
        error: {
          status: 'error',
          message: `Failed to update route: ${error instanceof Error ? error.message : String(error)}`,
        },
      }
    }
  },

  // Delete a route
  async deleteRoute(
    id: string
  ): Promise<ApiResponse<{ status: string; message: string }>> {
    try {
      const response = await fetch(`${API_URL}/routes/${id}`, {
        ...fetchOptions,
        method: 'DELETE',
      })
      return handleResponse<{ status: string; message: string }>(response)
    } catch (error) {
      return {
        error: {
          status: 'error',
          message: `Failed to delete route: ${error instanceof Error ? error.message : String(error)}`,
        },
      }
    }
  },
}
