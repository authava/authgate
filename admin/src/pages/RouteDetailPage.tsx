import { useState } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { api } from '../api/client';
import { Layout } from '../components/Layout';
import { LoadingSpinner } from '../components/LoadingSpinner';
import { ErrorMessage } from '../components/ErrorMessage';
import { ConfirmDialog } from '../components/ConfirmDialog';

export function RouteDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  // State for the delete confirmation dialog
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);

  // Fetch route details
  const {
    data: route,
    isLoading,
    isError,
    error
  } = useQuery({
    queryKey: ['route', id],
    queryFn: async () => {
      if (!id) throw new Error('Route ID is required');

      const response = await api.getRoute(id);
      if (response.error) {
        throw new Error(response.error.message);
      }
      return response.data;
    },
    enabled: !!id,
  });

  // Handle delete button click
  const handleDeleteClick = () => {
    setIsDeleteDialogOpen(true);
  };

  // Handle delete confirmation
  const handleDeleteConfirm = async () => {
    if (!id) return;

    try {
      const response = await api.deleteRoute(id);
      if (response.error) {
        throw new Error(response.error.message);
      }

      // Navigate back to the routes list
      navigate('/routes');
    } catch (err) {
      console.error('Failed to delete route:', err);
      setIsDeleteDialogOpen(false);
      // In a real app, you might want to show a toast or error message
    }
  };

  // Handle delete cancellation
  const handleDeleteCancel = () => {
    setIsDeleteDialogOpen(false);
  };

  // Render loading state
  if (isLoading) {
    return (
      <Layout>
        <div className="flex flex-col items-center justify-center h-64">
          <LoadingSpinner size="large" />
          <p className="mt-4 text-gray-600">Loading route details...</p>
        </div>
      </Layout>
    );
  }

  // Render error state
  if (isError || !route) {
    return (
      <Layout>
        <div className="max-w-4xl mx-auto">
          <ErrorMessage message={error?.message || 'Failed to load route details'} />
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
    );
  }

  return (
    <Layout>
      <div className="max-w-4xl mx-auto">
        <div className="flex justify-between items-center mb-6">
          <h1 className="text-2xl font-bold">Route Details</h1>
          <div className="flex space-x-4">
            <Link to="/routes" className="btn btn-secondary">
              Back to List
            </Link>
            <Link to={`/routes/${id}/edit`} className="btn btn-primary">
              Edit Route
            </Link>
            <button
              className="btn btn-danger"
              onClick={handleDeleteClick}
            >
              Delete
            </button>
          </div>
        </div>

        <div className="bg-white shadow-md rounded-lg overflow-hidden">
          <div className="px-6 py-4 border-b border-gray-200">
            <h2 className="text-xl font-semibold">Basic Information</h2>
          </div>

          <div className="px-6 py-4 grid grid-cols-2 gap-4">
            <div>
              <p className="text-sm font-medium text-gray-500">Host</p>
              <p className="mt-1 text-lg">{route.host}</p>
            </div>
            <div>
              <p className="text-sm font-medium text-gray-500">Path</p>
              <p className="mt-1 text-lg">{route.path}</p>
            </div>
            {route.id && (
              <div className="col-span-2">
                <p className="text-sm font-medium text-gray-500">ID</p>
                <p className="mt-1 text-sm font-mono bg-gray-100 p-1 rounded">{route.id}</p>
              </div>
            )}
          </div>

          <div className="px-6 py-4 border-t border-b border-gray-200">
            <h2 className="text-xl font-semibold">Requirements</h2>
          </div>

          <div className="px-6 py-4">
            {/* Roles */}
            <div className="mb-6">
              <h3 className="text-lg font-medium mb-2">Roles</h3>
              {route.require.roles && route.require.roles.length > 0 ? (
                <div className="flex flex-wrap gap-2">
                  {route.require.roles.map((role) => (
                    <span
                      key={role}
                      className="bg-blue-100 text-blue-800 px-3 py-1 rounded-full"
                    >
                      {role}
                    </span>
                  ))}
                </div>
              ) : (
                <p className="text-gray-500 italic">No roles specified</p>
              )}
            </div>

            {/* Permissions */}
            <div className="mb-6">
              <h3 className="text-lg font-medium mb-2">Permissions</h3>
              {route.require.permissions && route.require.permissions.length > 0 ? (
                <div className="flex flex-wrap gap-2">
                  {route.require.permissions.map((permission) => (
                    <span
                      key={permission}
                      className="bg-green-100 text-green-800 px-3 py-1 rounded-full"
                    >
                      {permission}
                    </span>
                  ))}
                </div>
              ) : (
                <p className="text-gray-500 italic">No permissions specified</p>
              )}
            </div>

            {/* Scopes */}
            <div className="mb-6">
              <h3 className="text-lg font-medium mb-2">Scopes</h3>
              {route.require.scopes && route.require.scopes.length > 0 ? (
                <div className="space-y-2">
                  {route.require.scopes.map((scope, index) => (
                    <div key={index} className="bg-purple-100 text-purple-800 px-3 py-2 rounded-md">
                      <span className="font-medium">Resource Type:</span> {scope.resource_type}<br />
                      <span className="font-medium">Action:</span> {scope.action}
                      {scope.resource_id && (
                        <><br /><span className="font-medium">Resource ID:</span> {scope.resource_id}</>
                      )}
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-gray-500 italic">No scopes specified</p>
              )}
            </div>

            {/* Teams */}
            <div>
              <h3 className="text-lg font-medium mb-2">Teams</h3>
              {route.require.teams && route.require.teams.length > 0 ? (
                <div className="space-y-2">
                  {route.require.teams.map((team, index) => (
                    <div key={index} className="bg-yellow-100 text-yellow-800 px-3 py-2 rounded-md">
                      {team.id && <div><span className="font-medium">ID:</span> {team.id}</div>}
                      {team.name && <div><span className="font-medium">Name:</span> {team.name}</div>}
                      {team.scopes && team.scopes.length > 0 && (
                        <div className="mt-2">
                          <span className="font-medium">Scopes:</span>
                          <div className="pl-4 mt-1 space-y-1">
                            {team.scopes.map((scope, scopeIndex) => (
                              <div key={scopeIndex}>
                                {scope.resource_type}:{scope.action}
                                {scope.resource_id && `:${scope.resource_id}`}
                              </div>
                            ))}
                          </div>
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-gray-500 italic">No teams specified</p>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Delete confirmation dialog */}
      <ConfirmDialog
        isOpen={isDeleteDialogOpen}
        title="Delete Route"
        message={
          <p>
            Are you sure you want to delete the route for
            <span className="font-semibold"> {route.host}{route.path}</span>?
            This action cannot be undone.
          </p>
        }
        confirmLabel="Delete"
        cancelLabel="Cancel"
        onConfirm={handleDeleteConfirm}
        onCancel={handleDeleteCancel}
      />
    </Layout>
  );
}