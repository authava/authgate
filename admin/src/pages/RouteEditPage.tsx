import { useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useQuery, useMutation } from '@tanstack/react-query'
import { api } from '../api/client'
import { Layout } from '../components/Layout'
import { RouteForm } from '../components/RouteForm'
import { LoadingSpinner } from '../components/LoadingSpinner'
import { ErrorMessage } from '../components/ErrorMessage'
import type { Route } from '../types'

export function RouteEditPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [error, setError] = useState<string | null>(null)

  // Fetch route details
  const {
    data: route,
    isLoading,
    isError,
    error: fetchError,
  } = useQuery({
    queryKey: ['route', id],
    queryFn: async () => {
      if (!id) throw new Error('Route ID is required')

      const response = await api.getRoute(id)
      if (response.error) {
        throw new Error(response.error.message)
      }
      return response.data
    },
    enabled: !!id,
  })

  // Update route mutation
  const updateRouteMutation = useMutation({
    mutationFn: async (updatedRoute: Route) => {
      if (!id) throw new Error('Route ID is required')

      const response = await api.updateRoute(id, updatedRoute)
      if (response.error) {
        throw new Error(response.error.message)
      }
      return response.data
    },
    onSuccess: () => {
      // Navigate to the route detail page
      navigate(`/routes/${id}`)
    },
    onError: (err: Error) => {
      setError(err.message)
      window.scrollTo(0, 0)
    },
  })

  // Handle form submission
  const handleSubmit = (values: Route) => {
    setError(null)
    updateRouteMutation.mutate(values)
  }

  // Render loading state
  if (isLoading) {
    return (
      <Layout>
        <div className="flex flex-col items-center justify-center h-64">
          <LoadingSpinner size="large" />
          <p className="mt-4 text-gray-600">Loading route details...</p>
        </div>
      </Layout>
    )
  }

  // Render error state
  if (isError || !route) {
    return (
      <Layout>
        <div className="max-w-4xl mx-auto">
          <ErrorMessage
            message={fetchError?.message || 'Failed to load route details'}
          />
          <div className="mt-4 flex space-x-4">
            <button
              className="btn btn-secondary"
              onClick={() => navigate('/routes')}
            >
              Back to Routes
            </button>
          </div>
        </div>
      </Layout>
    )
  }

  return (
    <Layout>
      <div className="max-w-3xl mx-auto">
        <div className="flex justify-between items-center mb-6">
          <h1 className="text-2xl font-bold">Edit Route</h1>
          <div className="flex space-x-4">
            <button
              className="btn btn-secondary"
              onClick={() => navigate(`/routes/${id}`)}
            >
              Cancel
            </button>
          </div>
        </div>

        {error && <ErrorMessage message={error} />}

        <div className="bg-white shadow-md rounded-lg p-6">
          <RouteForm
            initialValues={route}
            onSubmit={handleSubmit}
            isLoading={updateRouteMutation.isPending}
          />
        </div>
      </div>
    </Layout>
  )
}
