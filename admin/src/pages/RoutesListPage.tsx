import { useState } from 'react'
import { Link } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { api } from '../api/client'
import { Layout } from '../components/Layout'
import { LoadingSpinner } from '../components/LoadingSpinner'
import { ErrorMessage } from '../components/ErrorMessage'
import { ConfirmDialog } from '../components/ConfirmDialog'
import type { Route } from '../types'

export function RoutesListPage() {
  // State for the delete confirmation dialog
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false)
  const [routeToDelete, setRouteToDelete] = useState<Route | null>(null)

  // Fetch routes
  const {
    data: routes,
    isLoading,
    isError,
    error,
    refetch,
  } = useQuery({
    queryKey: ['routes'],
    queryFn: async () => {
      const response = await api.getRoutes()
      if (response.error) {
        throw new Error(response.error.message)
      }
      return response.data || []
    },
  })

  // Handle delete button click
  const handleDeleteClick = (route: Route) => {
    setRouteToDelete(route)
    setIsDeleteDialogOpen(true)
  }

  // Handle delete confirmation
  const handleDeleteConfirm = async () => {
    if (!routeToDelete?.id) return

    try {
      const response = await api.deleteRoute(routeToDelete.id)
      if (response.error) {
        throw new Error(response.error.message)
      }

      // Close the dialog and refetch routes
      setIsDeleteDialogOpen(false)
      setRouteToDelete(null)
      refetch()
    } catch (err) {
      console.error('Failed to delete route:', err)
      // Keep the dialog open but show an error
      // In a real app, you might want to show a toast or error message
    }
  }

  // Handle delete cancellation
  const handleDeleteCancel = () => {
    setIsDeleteDialogOpen(false)
    setRouteToDelete(null)
  }

  // Render loading state
  if (isLoading) {
    return (
      <Layout>
        <div className="flex flex-col items-center justify-center h-64">
          <LoadingSpinner size="large" />
          <p className="mt-4 text-gray-600">Loading routes...</p>
        </div>
      </Layout>
    )
  }

  // Render error state
  if (isError) {
    return (
      <Layout>
        <div className="max-w-4xl mx-auto">
          <ErrorMessage message={error?.message || 'Failed to load routes'} />
          <button className="btn btn-primary mt-4" onClick={() => refetch()}>
            Try Again
          </button>
        </div>
      </Layout>
    )
  }

  return (
    <Layout>
      <div className="max-w-6xl mx-auto">
        <div className="flex justify-between items-center mb-6">
          <h1 className="text-2xl font-bold">Routes</h1>
          <Link to="/routes/new" className="btn btn-primary">
            Create New Route
          </Link>
        </div>

        {routes && routes.length > 0 ? (
          <div className="bg-white shadow-md rounded-lg overflow-hidden">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Host
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Path
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Requirements
                  </th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {routes.map((route) => (
                  <tr key={route.id} className="hover:bg-gray-50">
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="text-sm font-medium text-gray-900">
                        {route.host}
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="text-sm text-gray-900">{route.path}</div>
                    </td>
                    <td className="px-6 py-4">
                      <div className="text-sm text-gray-500">
                        {route.require.roles &&
                          route.require.roles.length > 0 && (
                            <div className="mb-1">
                              <span className="font-medium">Roles:</span>{' '}
                              {route.require.roles.join(', ')}
                            </div>
                          )}
                        {route.require.permissions &&
                          route.require.permissions.length > 0 && (
                            <div className="mb-1">
                              <span className="font-medium">Permissions:</span>{' '}
                              {route.require.permissions.join(', ')}
                            </div>
                          )}
                        {route.require.scopes &&
                          route.require.scopes.length > 0 && (
                            <div className="mb-1">
                              <span className="font-medium">Scopes:</span>{' '}
                              {route.require.scopes.length} defined
                            </div>
                          )}
                        {route.require.teams &&
                          route.require.teams.length > 0 && (
                            <div>
                              <span className="font-medium">Teams:</span>{' '}
                              {route.require.teams.length} defined
                            </div>
                          )}
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                      <Link
                        to={`/routes/${route.id}`}
                        className="text-blue-600 hover:text-blue-900 mr-4"
                      >
                        View
                      </Link>
                      <Link
                        to={`/routes/${route.id}/edit`}
                        className="text-indigo-600 hover:text-indigo-900 mr-4"
                      >
                        Edit
                      </Link>
                      <button
                        onClick={() => handleDeleteClick(route)}
                        className="text-red-600 hover:text-red-900"
                      >
                        Delete
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <div className="bg-white shadow-md rounded-lg p-8 text-center">
            <p className="text-gray-500 mb-4">No routes found</p>
            <Link to="/routes/new" className="btn btn-primary">
              Create Your First Route
            </Link>
          </div>
        )}
      </div>

      {/* Delete confirmation dialog */}
      <ConfirmDialog
        isOpen={isDeleteDialogOpen}
        title="Delete Route"
        message={
          <p>
            Are you sure you want to delete the route for
            <span className="font-semibold">
              {' '}
              {routeToDelete?.host}
              {routeToDelete?.path}
            </span>
            ? This action cannot be undone.
          </p>
        }
        confirmLabel="Delete"
        cancelLabel="Cancel"
        onConfirm={handleDeleteConfirm}
        onCancel={handleDeleteCancel}
      />
    </Layout>
  )
}
